use crate::{BroadcastMessage, Connection, Shutdown};

use crate::db::Db;
use futures::StreamExt;
use std::future::Future;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::{self, Duration};
use tracing::{debug, error, info};

struct Listener {
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: broadcast::Sender<()>,
    broadcast_message: broadcast::Sender<BroadcastMessage>,
    shutdown_complete_rx: mpsc::Receiver<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

struct Handler {
    connection: Connection,
    db: Db,
    shutdown: Shutdown,
    _shutdown_complete: mpsc::Sender<()>,
}

const MAX_CONNECTIONS: usize = 100;

pub async fn run(listener: TcpListener, shutdown: impl Future) -> crate::Result<()> {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (broadcast_message, _) = broadcast::channel(100);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    let mut server = Listener {
        listener,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        notify_shutdown,
        broadcast_message,
        shutdown_complete_tx,
        shutdown_complete_rx,
    };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
        _ = shutdown => {
            info!("shutting down");
        }
    }

    let Listener {
        mut shutdown_complete_rx,
        shutdown_complete_tx,
        notify_shutdown,
        ..
    } = server;

    drop(notify_shutdown);
    drop(shutdown_complete_tx);

    let _ = shutdown_complete_rx.recv().await;

    Ok(())
}

impl Listener {
    async fn run(&mut self) -> crate::Result<()> {
        info!("accepting inbound connections");
        let db = Db::new();
        loop {
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            let socket = self.accept().await?;
            let message_sender:
                broadcast::Sender<BroadcastMessage>
             =
                self.broadcast_message.clone();

            let mut handler = Handler {
                connection: Connection::new(socket, message_sender),
                db: db.clone(),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            info!("Created new handler");

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    error!(cause = ?err, "connection error");
                }
                drop(permit);
            });
        }
    }

    async fn accept(&mut self) -> crate::Result<TcpStream> {
        let mut backoff = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into());
                    }
                }
            }

            time::sleep(Duration::from_secs(backoff)).await;

            backoff *= 2;
        }
    }
}

impl Handler {
    async fn run(&mut self) -> crate::Result<()> {
        let welcome = String::from("Welcome to budgetchat! What shall I call you?");
        let username;

        // Send the Welcome message to the connected client
        let _ = self.connection.write_frame(welcome).await;

        // Read the answer (username) from the client
        if let Some(Ok(name)) = self.connection.stream.next().await {
            info!("Add {name} to db");
            self.db.insert_user(name.clone()).await?;
            username = name;
        } else {
            return Ok(());
        }

        // Broadcast the message "* USER has entered the room"
        let joined_message = format!("* {username} has entered the room");
        let _ = self.connection
            .broadcast_message(BroadcastMessage::new(username.clone(), joined_message));

        // Write back directly to the client which users are currently in the room
        let room_contains_message = format!(
            "* The room contains {}",
            self.db.get_room_members(username.clone()).await.join(",")
        );
        let _ = self.connection.write_frame(room_contains_message).await;

        // Connect the client to the broadcast channel
        let mut receiver = self.connection.broadcast.subscribe();

        while !self.shutdown.is_shutdown() {
            tokio::select! {
                res = self.connection.stream.next() => match res {
                    Some(Ok(frame)) => {
                        let _ = self.connection
                            .broadcast_message(BroadcastMessage::new(username.clone(), format!("[{username}] {frame}")));
                    },
                    Some(Err(_)) => {
                        error!("Could not parse frame");
                        continue;
                    },
                    None => {
                        let message = format!("* {username} has left the room");
                        let _ = self.connection.broadcast_message(BroadcastMessage::new(username.clone(), message.clone()));
                        let _ = self.db.remove(username).await;
                        return Ok(())
                    },
                },
                message = receiver.recv() => {
                    info!("Message received: {:?}", message.as_ref().unwrap());
                    if message.as_ref().unwrap().from != username {
                        let _ = self.connection.write_frame(message.as_ref().unwrap().message.clone()).await;
                    }
                }
                _ = self.shutdown.recv() => {
                    debug!("Shutdown");
                    return Ok(());
                }
            };
        }

        Ok(())
    }
}
