use std::collections::HashMap;
use std::sync::Mutex;
use std::{io, net::SocketAddr, str, sync::Arc};
use tokio::{net::UdpSocket, sync::mpsc};

#[tokio::main]
async fn main() -> io::Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:8080".parse::<SocketAddr>().unwrap()).await?;
    let r = Arc::new(sock);
    let s = r.clone();
    let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);
    let storage = Arc::new(Mutex::new(HashMap::<String, String>::new()));

    tokio::spawn(async move {
        while let Some((bytes, addr)) = rx.recv().await {
            let len = s.send_to(&bytes, &addr).await.unwrap();
        }
    });

    let mut buf = [0; 1024];
    loop {
        let (len, addr) = r.recv_from(&mut buf).await?;
        let message = str::from_utf8(&buf[..len]).unwrap().trim_matches('\n');

        if message.contains("=") {
            let (key, value) = message.split_once('=').unwrap();

            storage
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            // continue;
        } else {
            let value = storage.lock().unwrap().get(message).unwrap().clone();
            tx.send((value.as_bytes().to_vec(), addr)).await.unwrap();
        }

        buf.fill(0);
    }
}
