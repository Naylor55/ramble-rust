//! RTMP Streaming Server Library
//!
//! This crate provides a simple RTMP streaming server implementation.

mod error;
mod protocol;
mod server;
mod session;
mod stream;

pub use error::{Error, Result};
pub use server::RtmpServer;
pub use stream::{Stream, StreamManager};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default RTMP port
pub const DEFAULT_RTMP_PORT: u16 = 1935;

/// Maximum number of concurrent connections
pub const MAX_CONNECTIONS: usize = 1000;

/// Maximum stream buffer size in bytes
pub const MAX_STREAM_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10MB