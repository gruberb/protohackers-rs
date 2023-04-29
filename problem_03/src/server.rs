use futures::{stream::StreamExt, SinkExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::{
    broadcast,
    broadcast::{Receiver, Sender},
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{error, info};

const IP: &str = "0.0.0.0";
const PORT: u16 = 1222;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

type Username = String;
type Message = String;
type Address = SocketAddr;

#[derive(Clone, Debug, Default)]
struct BroadcastMessage(Username, Message);

#[derive(Clone, Debug, Default)]
struct Users(Arc<Mutex<HashMap<Username, Address>>>);

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::try_init().expect("Tracing was not setup");

    let listener = TcpListener::bind(format!("{IP}:{PORT}")).await?;
    info!("Listening on: {}", format!("{IP}:{PORT}"));

    let (tx, _) = broadcast::channel(256);

    let db = Users::default();

    // Infinite loop to always listen to new connections on this IP/PORT
    loop {
        let (stream, address) = listener.accept().await?;
        let (tx, mut rx) = (tx.clone(), tx.subscribe());
        let db = db.clone();

        tokio::spawn(async move {
            let mut framed = Framed::new(stream, LinesCodec::new());
            info!("New address connected: {address}");
            let _ = framed
                .send("Welcome to budgetchat! What shall I call you?".to_string())
                .await;

            let mut name = String::default();

            // We read exactly one line per loop. A line ends with \n.
            // So if the client doesn't frame their package with \n at the end,
            // we won't process until we find one.
            match framed.next().await {
                Some(Ok(username)) => {
                    name = username.clone();
                    db.0.lock().unwrap().insert(username.clone(), address);
                    let message = compose_message(username.clone(), db.clone());
                    info!("Adding username: {username} to db");
                    let _ = framed.send(message).await;
                    info!("Send message to client");
                    let b = BroadcastMessage(
                        username.clone(),
                        format!("* {} has entered the room", username),
                    );
                    let _ = tx.send(b);
                }
                Some(Err(e)) => {
                    error!("Error parsing message: {e}");
                }
                None => {
                    info!("No frame");
                }
            }

            loop {
                tokio::select! {
                    n = framed.next() => {
                        match n {
                            Some(Ok(n)) => {
                                // broadcast message to all clients except the one who sent it
                                info!("Receiving new chat message: {n}");
                                let b =
                                    BroadcastMessage(name.clone(), format!("[{}]: {}", name, n));
                                let _ = tx.send(b);
                            }
                            Some(Err(e)) => {
                                error!("Error receiving chat message: {e}");
                            }
                            None => {
                                // Connection dropped
                                // remove client from db etc.
                                // send leave message
                                info!("No next frame");
                                let b =
                                    BroadcastMessage(name.clone(), format!("* {} has left the room", name));
                                db.0.lock().unwrap().remove(&name.clone());
                                let _ = tx.send(b);
                                break;
                            }
                        }
                    }
                    message = rx.recv() => {
                        let broadcast = message.clone().unwrap();
                        info!("Broadcast received: {:?}", message.clone().unwrap());
                        if broadcast.0 != name {
                            info!("Broadcast sent to {}: {:?}", name, message.clone().unwrap());
                            let _ = framed.send(message.unwrap().1).await;
                        }

                    }
                }
            }
        });
    }
}

fn compose_message(name: String, db: Users) -> String {
    format!(
        "* The room contains: {}",
        db.0.lock()
            .unwrap()
            .keys()
            .filter(|n| n.as_str() != name)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}
