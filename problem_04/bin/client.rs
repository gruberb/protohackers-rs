use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;

const MAX_DATAGRAM_SIZE: usize = 1000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} <server-ip:port> <request>", args[0]);
        return Ok(());
    }

    let server_addr: SocketAddr = args[1].parse()?;
    let request = args[2].clone();

    let local_addr = if server_addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };

    let socket = UdpSocket::bind(local_addr).await?;
    socket.connect(server_addr).await?;

    socket.send(request.as_bytes()).await?;

    if request.contains('=') {
        println!("Insert request sent. No response expected.");
    } else {
        let mut buf = vec![0; MAX_DATAGRAM_SIZE];
        let _ = tokio::time::timeout(Duration::from_secs(1), socket.recv(&mut buf)).await?;

        let response = String::from_utf8_lossy(&buf);
        println!("Received response: {}", response);
    }

    Ok(())
}
