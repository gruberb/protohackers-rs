mod connection;
pub use connection::Connection;

pub mod frame;
pub use frame::Frame;

pub mod server;

mod shutdown;
use shutdown::Shutdown;

pub const DEFAULT_PORT: u16 = 6379;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
