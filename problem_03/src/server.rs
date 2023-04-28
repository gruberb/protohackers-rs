use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::TcpListener,
};
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
        let (mut stream, address) = listener.accept().await?;
        info!("New address connected: {}", address);
        // We spawn a new task, so every incoming connection can be put on a thread
        // and be worked on "in the background"
        // This allows us to handle multiple connections "at the same time"
        let _ = stream.write_all("You are connected!\n".as_bytes()).await;
        tokio::spawn(async move {
            // From the stream (TcpStream), we can extract the reading, and the writing part
            // So we can read and write to the connected client on this port
            let (reader, writer) = stream.split();

            // So we don't read "directly" on the reader. Therefore we use
            // BufReader, which performs large, infrequent reads on the underlying
            // AsyncRead instance (reader)
            let mut reader = BufReader::new(reader);

            // We do the same for the writing part to the stream
            // let mut writer = BufWriter::new(writer);
            let mut writer = BufWriter::new(writer);

            // We need to store what we read from the stream in a local buffer/object
            let mut line = String::new();

            loop {
                // We read exactly one line per loop. A line ends with \n.
                // So if the client doesn't frame their package with \n at the end,
                // we won't process until we find one.
                let _ = match reader.read_line(&mut line).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        error!("Error reading: {}", e);
                        return;
                    }
                };

                info!("New client message received: {}", line.trim_end());

                if let Err(e) = writer.write_all(line.as_bytes()).await {
                    error!("Error writing: {}", e);
                    return;
                }

                let _ = writer.write_all(&[b'\n']).await;
                let _ = writer.flush().await;

                line.clear();
            }
        });
    }
}
