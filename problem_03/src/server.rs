use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{error, info};

const IP: &str = "0.0.0.0";
const PORT: u16 = 1222;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::try_init().expect("Tracing was not setup");

    let listener = TcpListener::bind(format!("{IP}:{PORT}")).await?;
    info!("Listening on: {}", format!("{IP}:{PORT}"));
    // Infinite loop to always listen to new connections on this IP/PORT
    loop {
        // Get the TCP stream out of the new connection, and the address from which
        // it is connected to

        let (stream, address) = listener.accept().await?;
        let mut framed = Framed::new(stream, LinesCodec::new());
        info!("New address connected: {}", address);
        let _ = framed.send("You are connected!".to_string()).await;

        // We spawn a new task, so every incoming connection can be put on a thread
        // and be worked on "in the background"
        // This allows us to handle multiple connections "at the same time"
        tokio::spawn(async move {
            loop {
                // We read exactly one line per loop. A line ends with \n.
                // So if the client doesn't frame their package with \n at the end,
                // we won't process until we find one.
                match framed.next().await {
                    Some(n) => {
                        if let Err(e) = n {
                            error!("Error parsing message: {}", e);
                        } else {
                            let _ = framed.send(n.unwrap()).await;
                        }
                    }
                    None => return,
                };
            }
        });
    }
}
