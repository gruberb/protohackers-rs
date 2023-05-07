use problem_06::{server, DEFAULT_IP, DEFAULT_PORT};

use tokio::net::TcpListener;
use tokio::signal;

#[tokio::main]
pub async fn main() -> problem_06::Result<()> {
    tracing_subscriber::fmt::try_init().expect("Couldn't setup logging");

    // Bind a TCP listener
    let listener = TcpListener::bind(&format!("{DEFAULT_IP}:{DEFAULT_PORT}")).await?;

    let _ = server::run(listener, signal::ctrl_c()).await;

    Ok(())
}
