//! RTMP server implementation

use crate::{error::{Error, Result}, session::RtmpSession, stream::StreamManager};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, error, warn};

/// RTMP Server
pub struct RtmpServer {
    /// Address to bind to
    address: SocketAddr,
    /// Stream manager
    stream_manager: StreamManager,
    /// Maximum connections
    max_connections: usize,
}

impl RtmpServer {
    /// Create a new RTMP server
    pub fn new(address: SocketAddr) -> Self {
        Self {
            address,
            stream_manager: StreamManager::new(),
            max_connections: 1000,
        }
    }

    /// Set maximum connections
    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = max_connections;
        self
    }

    /// Run the server
    pub async fn run(self) -> Result<()> {
        info!("Starting RTMP server on {}", self.address);

        let listener = TcpListener::bind(self.address).await?;
        info!("Server listening on {}", self.address);

        // TODO: Implement connection limiting
        // TODO: Implement graceful shutdown

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    info!("New connection from {}", addr);

                    // TODO: Spawn a task to handle the connection
                    // For now, just accept and close
                    let _ = socket;
                    warn!("Connection handling not implemented yet");
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address
    pub address: SocketAddr,
    /// Maximum connections
    pub max_connections: usize,
    /// Stream buffer size
    pub stream_buffer_size: usize,
    /// Enable logging
    pub enable_logging: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: format!("0.0.0.0:{}", crate::DEFAULT_RTMP_PORT)
                .parse()
                .unwrap(),
            max_connections: crate::MAX_CONNECTIONS,
            stream_buffer_size: crate::MAX_STREAM_BUFFER_SIZE,
            enable_logging: true,
        }
    }
}