use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    log::info!("Start TCP server");
    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => {
                        log::info!("Receiving echo: {}", n);
                        return;
                    }
                    Ok(n) => {
                        log::info!("Receiving echo: {}", n);
                        n
                    }
                    Err(e) => {
                        log::error!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    log::error!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
