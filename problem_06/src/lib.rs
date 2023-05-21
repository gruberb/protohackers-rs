mod connection;
mod db;
mod frame;
mod heartbeat;
pub mod server;
mod shutdown;
mod ticketing;

pub use connection::Connection;
pub use frame::ClientFrames;
use shutdown::Shutdown;

pub const DEFAULT_IP: &'static str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 1222;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
