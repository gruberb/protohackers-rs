use fancy_regex::Regex;
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tracing::{error, info};

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
    let mut framed_server_read = FramedRead::new(server_read, LinesCodec::new());
    let mut framed_server_write = FramedWrite::new(server_write, LinesCodec::new());

    loop {
        tokio::select! {
            res = framed_client_read.next() => {
                info!("Response from client read: {:?}", res);
                match res {
                    Some(response) => {
                        match response {
                            Ok(message) => {
                                info!("Send upstream: {message}");
                                let _ = framed_server_write.send(replace_address(message)).await;
                            }
                            Err(err) => {
                                error!("Error reading from client: {err}");
                                return Err(err.into());
                            }
                        }
                    }
                    None => {
                        info!("Client closed the connection");
                        break;
                    }
                }
            }
            res = framed_server_read.next() => {
                info!("Response from server read: {:?}", res);
                match res {
                    Some(response) => {
                        match response {
                            Ok(message) => {
                                info!("Send to client: {message}");
                                let _ = framed_client_write.send(replace_address(message)).await;
                            }
                            Err(err) => {
                                error!("Error reading from server: {err}");
                                return Err(err.into());
                            }
                        }
                    }
                    None => {
                        info!("Server closed the connection");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn replace_address(message: String) -> String {
    let replacement = "7YWHMfk9JZe0LM0g1ZauHuiSxhI";
    let pattern = r"(?<= |^)7[a-zA-Z0-9]{25,34}(?= |$)";
    let re = Regex::new(pattern).unwrap();

    let res = re.replace_all(&message, replacement).to_string();

    info!("Replaced message: {res}");

    res
}
