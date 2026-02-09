//! Stream management

use crate::error::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Stream data
#[derive(Debug, Clone)]
pub struct StreamData {
    /// Stream name
    pub name: String,
    /// Stream key (for publishing)
    pub key: Option<String>,
    /// Stream metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: std::time::SystemTime,
    /// Last activity timestamp
    pub last_activity: std::time::SystemTime,
}

impl StreamData {
    /// Create a new stream
    pub fn new(name: String) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            name,
            key: None,
            metadata: HashMap::new(),
            created_at: now,
            last_activity: now,
        }
    }

    /// Update activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = std::time::SystemTime::now();
    }

    /// Check if stream is active (within timeout)
    pub fn is_active(&self, timeout: std::time::Duration) -> bool {
        match self.last_activity.elapsed() {
            Ok(elapsed) => elapsed < timeout,
            Err(_) => false,
        }
    }
}

/// Stream
pub struct Stream {
    /// Stream data
    data: Arc<RwLock<StreamData>>,
    /// Subscribers
    subscribers: Vec<String>, // TODO: Replace with actual subscriber connections
}

impl Stream {
    /// Create a new stream
    pub fn new(name: String) -> Self {
        Self {
            data: Arc::new(RwLock::new(StreamData::new(name))),
            subscribers: Vec::new(),
        }
    }

    /// Get stream name
    pub async fn name(&self) -> String {
        self.data.read().await.name.clone()
    }

    /// Add a subscriber
    pub async fn add_subscriber(&mut self, subscriber_id: String) {
        self.subscribers.push(subscriber_id);
        self.data.write().await.update_activity();
        debug!("Added subscriber to stream {}", self.data.read().await.name);
    }

    /// Remove a subscriber
    pub async fn remove_subscriber(&mut self, subscriber_id: &str) {
        self.subscribers.retain(|id| id != subscriber_id);
        self.data.write().await.update_activity();
        debug!("Removed subscriber from stream {}", self.data.read().await.name);
    }

    /// Get subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    /// Publish data to stream
    pub async fn publish(&mut self, data: &[u8]) -> Result<()> {
        // TODO: Implement actual data publishing
        let stream_name = self.data.read().await.name.clone();
        debug!("Publishing {} bytes to stream {}", data.len(), stream_name);
        self.data.write().await.update_activity();

        // TODO: Broadcast to subscribers
        Ok(())
    }

    /// Check if stream is active
    pub async fn is_active(&self, timeout: std::time::Duration) -> bool {
        self.data.read().await.is_active(timeout)
    }
}

/// Stream manager
pub struct StreamManager {
    /// Active streams
    streams: Arc<RwLock<HashMap<String, Stream>>>,
    /// Stream timeout
    stream_timeout: std::time::Duration,
}

impl StreamManager {
    /// Create a new stream manager
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            stream_timeout: std::time::Duration::from_secs(300), // 5 minutes
        }
    }

    /// Create a new stream
    pub async fn create_stream(&self, name: String) -> Result<()> {
        let mut streams = self.streams.write().await;

        if streams.contains_key(&name) {
            return Err(Error::Stream(format!("Stream '{}' already exists", name)));
        }

        let stream = Stream::new(name.clone());
        streams.insert(name.clone(), stream);

        info!("Created new stream: {}", name);
        Ok(())
    }

    /// Get a stream
    pub async fn get_stream(&self, name: &str) -> Option<Stream> {
        let streams = self.streams.read().await;
        streams.get(name).cloned()
    }

    /// Remove a stream
    pub async fn remove_stream(&self, name: &str) -> Result<()> {
        let mut streams = self.streams.write().await;

        if streams.remove(name).is_some() {
            info!("Removed stream: {}", name);
            Ok(())
        } else {
            Err(Error::Stream(format!("Stream '{}' not found", name)))
        }
    }

    /// List all streams
    pub async fn list_streams(&self) -> Vec<String> {
        let streams = self.streams.read().await;
        streams.keys().cloned().collect()
    }

    /// Clean up inactive streams
    pub async fn cleanup_inactive(&self) -> usize {
        let mut streams = self.streams.write().await;
        let initial_count = streams.len();

        streams.retain(|_, stream| {
            // TODO: Check if stream is active
            // For now, keep all streams
            true
        });

        let removed = initial_count - streams.len();
        if removed > 0 {
            info!("Cleaned up {} inactive streams", removed);
        }

        removed
    }

    /// Subscribe to a stream
    pub async fn subscribe(&self, stream_name: &str, subscriber_id: String) -> Result<()> {
        let mut streams = self.streams.write().await;

        if let Some(stream) = streams.get_mut(stream_name) {
            stream.add_subscriber(subscriber_id).await;
            Ok(())
        } else {
            Err(Error::Stream(format!("Stream '{}' not found", stream_name)))
        }
    }

    /// Unsubscribe from a stream
    pub async fn unsubscribe(&self, stream_name: &str, subscriber_id: &str) -> Result<()> {
        let mut streams = self.streams.write().await;

        if let Some(stream) = streams.get_mut(stream_name) {
            stream.remove_subscriber(subscriber_id).await;
            Ok(())
        } else {
            Err(Error::Stream(format!("Stream '{}' not found", stream_name)))
        }
    }

    /// Publish data to a stream
    pub async fn publish(&self, stream_name: &str, data: &[u8]) -> Result<()> {
        let mut streams = self.streams.write().await;

        if let Some(stream) = streams.get_mut(stream_name) {
            stream.publish(data).await
        } else {
            Err(Error::Stream(format!("Stream '{}' not found", stream_name)))
        }
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Stream {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            subscribers: self.subscribers.clone(),
        }
    }
}