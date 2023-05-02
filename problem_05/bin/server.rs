use tokio::net::{TcpListener, TcpStream};
use tracing::{info, error};
use std::net::SocketAddr;
use tokio_util::codec::{Framed, LinesCodec};
use tokio::sync::broadcast;

const DEFAULT_IP: &str = "0.0.0.0";
const DEFAULT_PORT: &str = "1222";

const UPSTREAM_IP: &str = "206.189.113.124";
const UPSTREAM_PORT: &str = "16963";

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

enum Events {
    ClientRequest(String),
    ClientResponse(String),
    UpstreamRequest(String),
    UpstreamResponse(String),
}

#[tokio::main]
pub async fn main() -> Result<()> {
    tracing_subscriber::fmt::try_init()?;

    let listener = TcpListener::bind(&format!("{DEFAULT_IP}:{DEFAULT_PORT}")).await?;
    let stream = TcpStream::connect(&format!("{UPSTREAM_IP}:{UPSTREAM_PORT}")).await?;

    let (sender, receiver) = broadcast::channel(2);

    info!("Start TCP server on {DEFAULT_IP}:{DEFAULT_PORT}");
    info!("Connect to upstream on {UPSTREAM_IP}:{UPSTREAM_PORT}");

    let listener_handle = tokio::spawn(async move {
        loop {
            let (socket, address) = listener.accept().await?;

            tokio::spawn(async move {
                info!("New request from: {address}");
                let _ = handle_request(socket).await;
            });
        }
    });

    let upstream_handle = tokio::spawn({
        loop {

        }
    });

    let _ = listener_handle.await;
    let _ = upstream_handle.await;

    Ok(())

}

pub async fn handle_request(mut socket: TcpStream) -> Result<()> {
    let framed = Framed::new(socket, LinesCodec::new());
}
