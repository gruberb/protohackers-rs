use crate::{frame::Frame, Connection, Shutdown};

use std::collections::BTreeMap;
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
    shutdown_complete_rx: mpsc::Receiver<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

struct Handler {
    connection: Connection,
    shutdown: Shutdown,
    local_db: BTreeMap<Timestamp, Price>,
    _shutdown_complete: mpsc::Sender<()>,
}

type Timestamp = i32;
type Price = i32;

const MAX_CONNECTIONS: usize = 5;

pub async fn run(listener: TcpListener, shutdown: impl Future) -> crate::Result<()> {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    let mut server = Listener {
        listener,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        notify_shutdown,
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

        loop {
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            let socket = self.accept().await?;

            let mut handler = Handler {
                connection: Connection::new(socket),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                local_db: BTreeMap::new(),
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
        while !self.shutdown.is_shutdown() {
            let maybe_frame = tokio::select! {
                res = self.connection.read_frame() => res?,
                _ = self.shutdown.recv() => {
                    debug!("Shutdown");
                    return Ok(());
                }
            };

            debug!(?maybe_frame);

            let frame = match maybe_frame {
                Some(frame) => frame,
                None => return Ok(()),
            };

            match frame {
                Frame::Insert { timestamp, price } => {
                    self.local_db.insert(timestamp, price);
                }
                Frame::Query { mintime, maxtime } => {
                    debug!(?mintime, ?maxtime);

                    if mintime <= maxtime {
                        let mut count = 0;
                        let mut sum = 0i64;

                        for (_, price) in self.local_db.range(mintime..=maxtime) {
                            sum += *price as i64;
                            count += 1;
                        }

                        let mean = if count > 0 { sum / count } else { 0 };
                        debug!(?mean);
                        self.connection.write_frame(&Frame::Response(mean)).await?;
                    } else {
                        self.connection.write_frame(&Frame::Response(0)).await?;
                    }
                }
                _ => unimplemented!(),
            }
        }

        Ok(())
    }
}
