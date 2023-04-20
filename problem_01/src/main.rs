use primes::is_prime;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::{TcpListener, TcpStream};

const IS_PRIME: &str = "isPrime";
const MAL_FORMAT: &str = "}mal";

#[derive(Debug, Deserialize, Serialize)]
struct Request {
    method: String,
    number: serde_json::value::Number,
}

#[derive(Debug, Deserialize, Serialize)]
struct Response {
    method: String,
    prime: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let listener = TcpListener::bind("0.0.0.0:1222").await?;
    log::info!("Start TCP server");

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            log::info!("Handle incoming request");
            let _ = handle_request(socket).await;
        });
    }
}

async fn handle_request(mut socket: TcpStream) {
    let (read, mut write) = socket.split();

    let mut buf: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(read);

    loop {
        log::info!("Start a new loop!");
        let bytes = reader.read_until(b'\n', &mut buf).await;

        if let Ok(0) = bytes {
            log::info!("0 bytes sent");
            return;
        }

        let message: serde_json::Value = match serde_json::from_slice(&buf) {
            Ok(m) => m,
            Err(e) => {
                log::error!(
                    "Not a prober JSON message: {}",
                    e
                );
                let _ = write.write(&MAL_FORMAT.as_bytes()).await;
                let _ = write.write(&[b'\n']).await;
                let _ = write.flush().await;
                buf.clear();
                continue;
            }
        };

        log::info!("Message received: {}", message);

        match serde_json::from_slice::<Request>(&buf) {
            Ok(m) => {
                let possible_prime = match m.number.to_string().parse::<u64>() {
                    Ok(n) => n,
                    Err(e) => {
                        log::error!("Not a valid number: {}", e);
                        let message = serde_json::to_string(&Response {
                            method: IS_PRIME.to_owned(),
                            prime: false,
                        })
                        .unwrap();
                        log::info!("Sending back response: {}", message);
                        if let Err(e) = write.write(&message.as_bytes()).await {
                            log::error!("Error writing serialize step: {}", e);
                        }
                        if let Err(e) = write.write(&[b'\n']).await {
                            log::error!("Error writing: {}", e);
                        }
                        if let Err(e) = write.flush().await {
                            log::error!("Error flushing: {}", e);
                        }
                        buf.clear();
                        log::info!("After clearing buffer!");
                        continue;
                    }
                };
                log::info!("Number is valid!");
                let res = Response {
                    method: IS_PRIME.to_owned(),
                    prime: is_prime(possible_prime),
                };

                if m.method == IS_PRIME.to_owned() {
                    log::info!("All good, send response: {:?}", res);
                    if let Err(e) = write
                        .write(&serde_json::to_string(&res).unwrap().as_bytes())
                        .await
                    {
                        log::error!("Error writing serialize step: {}", e);
                    }
                    if let Err(e) = write.write(&[b'\n']).await {
                        log::error!("Error writing: {}", e);
                    }
                    if let Err(e) = write.flush().await {
                        log::error!("Error flushing: {}", e);
                    }
                    buf.clear();
                } else {
                    log::error!("Method is not isPrime");
                    if let Err(e) = write.write(&MAL_FORMAT.as_bytes()).await {
                        log::error!("Write mal_format failed: {}", e);
                    }

                    if let Err(e) = write.write(&[b'\n']).await {
                        log::error!("Error writing escape character: {}", e);
                    }
                    if let Err(e) = write.flush().await {
                        log::error!("Error flushing socket: {}", e);
                    }
                    log::info!("Wrote malformat response");
                    buf.clear();
                }
            }
            Err(e) => {
                log::error!("Error parsing the message: {}", e);
                log::error!("Message: {}", String::from_utf8_lossy(&buf));

                let _ = write.write(&MAL_FORMAT.as_bytes()).await;
                let _ = write.write(&[b'\n']).await;
                let _ = write.flush().await;
                buf.clear();
            }
        }
    }
}
