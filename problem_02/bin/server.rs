use problem_02::{server, DEFAULT_PORT};

use tokio::net::TcpListener;
use tokio::signal;

#[tokio::main]
pub async fn main() -> problem_02::Result<()> {
    tracing_subscriber::fmt::try_init()?;

    let listener = TcpListener::bind(&format!("0.0.0.0:{}", DEFAULT_PORT)).await?;

    server::run(listener, signal::ctrl_c()).await?;

    Ok(())
}
