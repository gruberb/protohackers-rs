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
                log::info!("Valid request");
                let _ = write.write(&m.as_bytes()).await;
            }
            Err(_) => {
                log::error!("Not valid request");
                let _ = write.write(&MAL_FORMAT.as_bytes()).await;
            }
        }

        let _ = write.write(&[b'\n']).await;
        let _ = write.flush().await;
        buf.clear();
    }
}

fn validate_request(message: Vec<u8>) -> Result<String, std::io::Error> {
    match serde_json::from_slice::<Request>(&message) {
        Ok(m) => {
            let possible_prime = match m.number.to_string().parse::<u64>() {
                Ok(n) => n,
                Err(_) => {
                    log::error!("Not a valid number for a prime candidate: {}", m.number);
                    return Ok(serde_json::to_string(&Response {
                        method: IS_PRIME.to_owned(),
                        prime: false,
                    })
                    .unwrap());
                }
            };

            if m.method == IS_PRIME.to_owned() {
                log::info!("Method isPrime and possible prime number");
                return Ok(serde_json::to_string(&Response {
                    method: IS_PRIME.to_owned(),
                    prime: is_prime(possible_prime),
                })
                .unwrap());
            } else {
                log::error!("Method is not isPrime");
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Method is not isPrime",
                ));
            }
        }
        Err(_) => {
            log::error!("Message is not a valid JSON or Request type");
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Message is not a Request",
            ));
        }
    }
}
