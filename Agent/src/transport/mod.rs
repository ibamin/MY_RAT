pub mod http;
pub mod protocol;

pub type TransportResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
