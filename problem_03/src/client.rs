use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::task;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::try_init().expect("Tracing was not setup");

    let stream = TcpStream::connect("0.0.0.0:1222").await?;

    let (reader, writer) = tokio::io::split(stream);
    let mut buf_reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    let server_handle = task::spawn(async move {
        let mut buf = String::new();

        loop {
            info!("Inside reading lines from server loop");
            if let Ok(n) = buf_reader.read_line(&mut buf).await {
                if n > 0 {
                    info!("Receivng from server: {}", buf.trim_end());
                } else {
                    info!("Server is finished sending, break");
                    return;
                }
            } else {
                error!("Cannot receive");
                return;
            }

            buf.clear();
        }
    });

    let std_handle = tokio::spawn(async move {
        let mut stdin_reader = BufReader::new(tokio::io::stdin()).lines();
        while let Ok(Some(line)) = stdin_reader.next_line().await {
            info!("Received line from stdin: {}", line);

            if let Err(_) = writer.write_all(line.as_bytes()).await {
                error!("Error reading from std");
                break;
            }

            let _ = writer.write_all(&[b'\n']).await;
            let _ = writer.flush().await;
        }
    });

    let _ = server_handle.await;
    let _ = std_handle.await;

    Ok(())
}
