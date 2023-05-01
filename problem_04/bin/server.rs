use std::collections::HashMap;
use std::{io, net::SocketAddr, str, sync::Arc};
use tokio::{net::UdpSocket, sync::mpsc};
use tracing::info;

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::try_init().unwrap();

    let sock = UdpSocket::bind("0.0.0.0:1222".parse::<SocketAddr>().unwrap()).await?;
    info!("listening to new connections");

    let r = Arc::new(sock);
    let s = r.clone();
    let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);
    let mut storage = HashMap::<String, String>::new();

    tokio::spawn(async move {
        info!("Send back!");
        while let Some((bytes, addr)) = rx.recv().await {
            let _ = s.send_to(&bytes, &addr).await.unwrap();
        }
    });

    let mut buf = [0; 1024];
    loop {
        info!("Recv loop");
        let (len, addr) = r.recv_from(&mut buf).await?;
        let message = str::from_utf8(&buf[..len]).unwrap().trim_matches('\n');

        if message.contains("version") {
            let message = format!("version=gruberb 1.0");
            tx.send((message.as_bytes().to_vec(), addr)).await.unwrap();
        } else if message.contains("=") {
            let (key, value) = message.split_once('=').unwrap();
            storage
                .insert(key.to_string(), value.to_string());
        } else {
            let value = storage.get(message).unwrap_or(&String::new()).clone();
            let message = format!("{message}={value}");
            tx.send((message.as_bytes().to_vec(), addr)).await.unwrap();
        }

        buf.fill(0);
    }
}
