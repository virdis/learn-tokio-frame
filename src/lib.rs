pub mod frame;
pub use frame::Frame;
pub mod connection;
pub use connection::Connection;

pub mod server;

pub mod clients;
pub use clients::Client;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
