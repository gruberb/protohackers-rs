use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc::channel;
use tokio::task;
use tracing::{debug, info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::try_init().expect("Tracing was not setup");

    let stream = TcpStream::connect("127.0.0.1:8080").await?;

    let (tx, mut rx) = channel::<String>(10);

    let (mut reader, mut writer) = tokio::io::split(stream);

    let tx_clone = tx.clone();

    task::spawn(async move {
        let mut reader = BufReader::new(&mut reader);

        loop {
            info!("Inside reading lines from server loop");
            let mut buf = String::new();
            if let Ok(n) = reader.read_line(&mut buf).await {
                if n > 0 {
                    println!("{}", buf.trim_end());
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        tx_clone.send("exit".to_string()).await.unwrap();
    });

    loop {
        info!("Inside read from std::io loop");
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;

        let buf = buf.trim_end().to_string();
        info!("New line: {}", buf);
        if buf.to_lowercase() == "exit" {
            break;
        }
        debug!(?buf);
        if let Err(_) = writer.write_all(buf.as_bytes()).await {
            error!("Could not sent");
            break;
        }

        if let Some(msg) = rx.recv().await {
            if msg.to_lowercase() == "exit" {
                break;
            }
        }
    }

    Ok(())
}
