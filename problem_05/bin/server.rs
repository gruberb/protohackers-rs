use futures::{SinkExt, StreamExt};
use regex::Regex;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tracing::info;

const DEFAULT_IP: &str = "0.0.0.0";
const DEFAULT_PORT: &str = "1222";

const UPSTREAM_IP: &str = "206.189.113.124";
const UPSTREAM_PORT: &str = "16963";

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
pub async fn main() -> Result<()> {
    tracing_subscriber::fmt::try_init().expect("Cannot init tracing");

    let listener = TcpListener::bind(&format!("{DEFAULT_IP}:{DEFAULT_PORT}")).await?;

    info!("Start TCP server on {DEFAULT_IP}:{DEFAULT_PORT}");

    loop {
        let (socket, address) = listener
            .accept()
            .await
            .expect("Cannot establish connection");

        info!("New request from: {address}");

        let upstream = TcpStream::connect(&format!("{UPSTREAM_IP}:{UPSTREAM_PORT}"))
            .await
            .expect("Cannot establish upstream connection");

        info!("Connect to upstream on {UPSTREAM_IP}:{UPSTREAM_PORT}");

        tokio::spawn(async move {
            let _ = handle_request(socket, upstream).await;
        });
    }
}

pub async fn handle_request(socket: TcpStream, upstream: TcpStream) -> Result<()> {
    let (client_read, client_write) = socket.into_split();
    let mut framed_client_read = FramedRead::new(client_read, LinesCodec::new());
    let mut framed_client_write = FramedWrite::new(client_write, LinesCodec::new());

    let (server_read, server_write) = upstream.into_split();
    let mut farmed_server_read = FramedRead::new(server_read, LinesCodec::new());
    let mut framed_server_write = FramedWrite::new(server_write, LinesCodec::new());

    let read_client_write_upstream = tokio::spawn(async move {
        while let Some(Ok(request)) = framed_client_read.next().await {
            info!("Send upstream: {request}");
            let _ = framed_server_write.send(replace_address(request)).await;
        }
    });

    let read_upstream_write_client = tokio::spawn(async move {
        while let Some(Ok(response)) = farmed_server_read.next().await {
            info!("Send to client: {response}");
            let _ = framed_client_write.send(replace_address(response)).await;
        }
    });

    let _ = read_client_write_upstream.await;
    let _ = read_upstream_write_client.await;

    Ok(())
}

fn replace_address(message: String) -> String {
    let replacement = "7YWHMfk9JZe0LM0g1ZauHuiSxhI";
    let pattern = r"\b(7[a-zA-Z0-9]{25,34})\b";
    let re = Regex::new(pattern).unwrap();

    let res = re.replace_all(&message, replacement).to_string();

    info!("Replaced message: {res}");

    res
}
