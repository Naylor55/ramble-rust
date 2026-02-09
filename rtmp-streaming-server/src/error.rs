//! Error types for the RTMP streaming server

use thiserror::Error;

/// Result type for the RTMP streaming server
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the RTMP streaming server
#[derive(Error, Debug)]
pub enum Error {
    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// Protocol errors
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Stream errors
    #[error("Stream error: {0}")]
    Stream(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid input errors
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    /// Internal errors
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Internal(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Internal(s)
    }
}