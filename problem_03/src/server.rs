use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast::{self, Sender};
use tracing::{debug, info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::try_init().expect("Tracing was not setup");

    let listener = TcpListener::bind("0.0.0.0:1222").await?;
    info!("Start listening on 0.0.0.0:1222");

    let clients = Arc::new(Mutex::new(HashMap::new()));
    let (tx, _) = broadcast::channel(10);

    loop {
        let (mut socket, _addr) = listener.accept().await?;
        let clients = clients.clone();
        let tx = tx.clone();
        // let mut stream = stream?;

        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.split();
            let mut buf = String::new();
            let mut reader = BufReader::new(&mut reader);

            // Request the user's name
            if let Err(e) = writer
                .write_all(b"Welcome to budgetchat! What shall I call you?\n")
                .await
            {
                println!("Failed to send name request: {}", e);
                return;
            }

            // Get the user's name
            match reader.read_line(&mut buf).await {
                Ok(_) => {
                    let name = buf.trim().to_string();
                    info!("Receiving name: {}", name);
                    if !is_valid_name(&name) {
                        if let Err(e) = writer
                            .write_all(b"Invalid name. Connection closed.\n")
                            .await
                        {
                            println!("Failed to send error message: {}", e);
                        }
                        return;
                    }

                    let (client_tx, _client_rx) = broadcast::channel(10);

                    {
                        let mut clients = clients.lock().unwrap();
                        announce_join(&name, &clients, &tx);
                        clients.insert(name.clone(), client_tx);
                    }

                    // Relay messages to other clients
                    while let Ok(_) = reader.read_line(&mut buf).await {
                        let message = buf.trim().to_string();

                        if message.is_empty() {
                            break;
                        }

                        relay_message(&name, &message, &tx);
                    }

                    // Client disconnected, remove from clients and announce leave
                    {
                        let mut clients = clients.lock().unwrap();
                        clients.remove(&name);
                        announce_leave(&name, &clients, &tx);
                    }
                }
                Err(e) => println!("Failed to read name: {}", e),
            }
        });
    }
}

fn is_valid_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric())
}

fn announce_join(name: &str, clients: &HashMap<String, Sender<String>>, tx: &Sender<String>) {
    let message = format!("* {} has entered the room", name);
    for client_name in clients.keys() {
        if client_name != name {
            let _ = tx.send(message.clone());
        }
    }
}

fn announce_leave(name: &str, clients: &HashMap<String, Sender<String>>, tx: &Sender<String>) {
    let message = format!("* {} has left the room", name);
    for client_name in clients.keys() {
        if client_name != name {
            let _ = tx.send(message.clone());
        }
    }
}

fn relay_message(name: &str, message: &str, tx: &Sender<String>) {
    let message = format!("[{}] {}", name, message);
    let _ = tx.send(message);
}
