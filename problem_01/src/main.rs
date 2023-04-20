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
    number: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Response {
    method: String,
    prime: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
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
    let bytes = reader.read_until(b'\n', &mut buf).await;

    if let Ok(0) = bytes {
        return;
    }

    if let Ok(1) = bytes {
        return;
    }

    match serde_json::from_slice::<Request>(&buf) {
        Ok(m) => {
            log::error!("Message received: {:?}", m);
            log::error!("Right method set? {}", m.method == IS_PRIME.to_owned());
            log::error!("Is {} a prime? {}", m.number, is_prime(m.number));

            let res = Response {
                method: IS_PRIME.to_owned(),
                prime: is_prime(m.number),
            };

            if m.method == IS_PRIME.to_owned() {
                let _ = write
                    .write(&bincode::serialize(&serde_json::to_string(&res).unwrap()).unwrap())
                    .await;
                let _ = write.write(&[b'\"', b'\n']).await;
                let _ = write.flush().await;
            } else {
                let _ = write.write(&bincode::serialize(&MAL_FORMAT).unwrap()).await;
                let _ = write.write(&[b'\"', b'\n']).await;
                let _ = write.flush().await;
            }
        }
        Err(e) => {
            log::error!("Error parsing the message: {}", e);
            log::error!("Message: {}", String::from_utf8_lossy(&buf));

            let _ = write.write(&bincode::serialize(&MAL_FORMAT).unwrap()).await;
            let _ = write.write(&[b'\"', b'\n']).await;
            let _ = write.flush().await;
        }
    }
}
