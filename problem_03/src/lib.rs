mod connection;

pub use connection::{BroadcastMessage, Connection};
use tokio::net::unix::SocketAddr;

pub mod server;

mod db;
mod shutdown;

use shutdown::Shutdown;

pub const DEFAULT_PORT: u16 = 1222;
pub const DEFAULT_IP: &str = "0.0.0.0";

pub type Username = String;
pub type Message = String;
pub type Address = SocketAddr;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
