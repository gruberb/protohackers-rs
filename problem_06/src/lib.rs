mod connection;
pub use connection::Connection;

pub mod frame;
pub use frame::Frame;

pub mod server;

mod shutdown;
use shutdown::Shutdown;

pub const DEFAULT_IP: &'static str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 1222;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
