use std::io::ErrorKind;

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
        let bytes = reader.read_until(b'\n', &mut buf).await;

        if let Ok(0) = bytes {
            log::info!("0 bytes sent");
            return;
        }

        match validate_request(buf.clone()) {
            Ok(m) => {
                let _ = write.write(&m.as_bytes()).await;
                let _ = write.write(&[b'\n']).await;
                let _ = buf.clear();
            }
            Err(_) => {
                let _ = write.write(&MAL_FORMAT.as_bytes()).await;
                let _ = write.write(&[b'\n']).await;
                let _ = write.flush().await;
                buf.clear();
            }
        }

        // Bad case
        // log::error!("Not a prober JSON message: {}", e);
        // let _ = write.write(&MAL_FORMAT.as_bytes()).await;
        // let _ = write.write(&[b'\n']).await;
        // let _ = write.flush().await;
        // buf.clear();

        // Good case
        // log::info!("Sending back response: {}", message);
        // if let Err(e) = write.write(&message.as_bytes()).await {
        //     log::error!("Error writing serialize step: {}", e);
        // }
        // if let Err(e) = write.write(&[b'\n']).await {
        //     log::error!("Error writing: {}", e);
        // }
        // if let Err(e) = write.flush().await {
        //     log::error!("Error flushing: {}", e);
        // }
        // buf.clear();
        // log::info!("After clearing buffer!");
    }
}

fn validate_request(message: Vec<u8>) -> Result<String, std::io::Error> {
    // Is it a proper formated JSON message?
    // Do I need this case?
    // let message: serde_json::Value = match serde_json::from_slice(&buf) {
    //     Ok(m) => m,
    //     Err(e) => {
    //         log::error!("Not a prober JSON message: {}", e);
    //         let _ = write.write(&MAL_FORMAT.as_bytes()).await;
    //         let _ = write.write(&[b'\n']).await;
    //         let _ = write.flush().await;
    //         buf.clear();
    //     }
    // };

    // Is it a proper Request?
    match serde_json::from_slice::<Request>(&message) {
        Ok(m) => {
            let possible_prime = match m.number.to_string().parse::<u64>() {
                Ok(n) => n,
                Err(_) => {
                    return Ok(serde_json::to_string(&Response {
                        method: IS_PRIME.to_owned(),
                        prime: false,
                    })
                    .unwrap());
                }
            };

            if m.method == IS_PRIME.to_owned() {
                return Ok(serde_json::to_string(&Response {
                    method: IS_PRIME.to_owned(),
                    prime: is_prime(possible_prime),
                })
                .unwrap());
            } else {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Method is not isPrime",
                ));
            }
        }
        Err(_) => {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Message is not a Request",
            ));
        }
    }
}
