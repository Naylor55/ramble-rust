//! RTMP protocol definitions and utilities

use crate::error::{Error, Result};

/// RTMP protocol version
pub const RTMP_VERSION: u8 = 3;

/// RTMP handshake size
pub const HANDSHAKE_SIZE: usize = 1536;

/// RTMP message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Set chunk size
    SetChunkSize = 1,
    /// Abort message
    AbortMessage = 2,
    /// Acknowledgement
    Acknowledgement = 3,
    /// User control message
    UserControl = 4,
    /// Window acknowledgement size
    WindowAcknowledgementSize = 5,
    /// Set peer bandwidth
    SetPeerBandwidth = 6,
    /// Audio data
    Audio = 8,
    /// Video data
    Video = 9,
    /// Data message (AMF3)
    DataAmf3 = 15,
    /// Shared object message (AMF3)
    SharedObjectAmf3 = 16,
    /// Command message (AMF3)
    CommandAmf3 = 17,
    /// Data message (AMF0)
    DataAmf0 = 18,
    /// Shared object message (AMF0)
    SharedObjectAmf0 = 19,
    /// Command message (AMF0)
    CommandAmf0 = 20,
    /// Aggregate message
    Aggregate = 22,
}

impl TryFrom<u8> for MessageType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            1 => Ok(MessageType::SetChunkSize),
            2 => Ok(MessageType::AbortMessage),
            3 => Ok(MessageType::Acknowledgement),
            4 => Ok(MessageType::UserControl),
            5 => Ok(MessageType::WindowAcknowledgementSize),
            6 => Ok(MessageType::SetPeerBandwidth),
            8 => Ok(MessageType::Audio),
            9 => Ok(MessageType::Video),
            15 => Ok(MessageType::DataAmf3),
            16 => Ok(MessageType::SharedObjectAmf3),
            17 => Ok(MessageType::CommandAmf3),
            18 => Ok(MessageType::DataAmf0),
            19 => Ok(MessageType::SharedObjectAmf0),
            20 => Ok(MessageType::CommandAmf0),
            22 => Ok(MessageType::Aggregate),
            _ => Err(Error::Protocol(format!("Invalid message type: {}", value))),
        }
    }
}

/// RTMP chunk header format
#[derive(Debug, Clone)]
pub enum ChunkHeaderFormat {
    /// 11 bytes: timestamp (3), message length (3), message type (1), stream id (4)
    Format0,
    /// 7 bytes: timestamp delta (3), message length (3), message type (1)
    Format1,
    /// 3 bytes: timestamp delta (3)
    Format2,
    /// 0 bytes (reuse previous header)
    Format3,
}

/// RTMP chunk header
#[derive(Debug, Clone)]
pub struct ChunkHeader {
    /// Basic header format
    pub format: ChunkHeaderFormat,
    /// Chunk stream ID
    pub chunk_stream_id: u32,
    /// Timestamp or timestamp delta
    pub timestamp: u32,
    /// Message length
    pub message_length: u32,
    /// Message type
    pub message_type: MessageType,
    /// Message stream ID
    pub message_stream_id: u32,
    /// Extended timestamp (if timestamp == 0xFFFFFF)
    pub extended_timestamp: Option<u32>,
}

impl ChunkHeader {
    /// Create a new chunk header
    pub fn new(
        format: ChunkHeaderFormat,
        chunk_stream_id: u32,
        timestamp: u32,
        message_length: u32,
        message_type: MessageType,
        message_stream_id: u32,
    ) -> Self {
        Self {
            format,
            chunk_stream_id,
            timestamp,
            message_length,
            message_type,
            message_stream_id,
            extended_timestamp: None,
        }
    }
}

/// RTMP message
#[derive(Debug, Clone)]
pub struct Message {
    /// Message type
    pub message_type: MessageType,
    /// Message stream ID
    pub message_stream_id: u32,
    /// Timestamp
    pub timestamp: u32,
    /// Payload
    pub payload: Vec<u8>,
}

impl Message {
    /// Create a new message
    pub fn new(
        message_type: MessageType,
        message_stream_id: u32,
        timestamp: u32,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            message_type,
            message_stream_id,
            timestamp,
            payload,
        }
    }
}

/// RTMP command types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandType {
    /// Connect command
    Connect,
    /// Call command
    Call,
    /// Create stream command
    CreateStream,
    /// Play command
    Play,
    /// Publish command
    Publish,
    /// Seek command
    Seek,
    /// Pause command
    Pause,
    /// Close stream command
    CloseStream,
    /// Delete stream command
    DeleteStream,
    /// Receive audio command
    ReceiveAudio,
    /// Receive video command
    ReceiveVideo,
    /// Release stream command
    ReleaseStream,
    /// FCPublish command
    FCPublish,
    /// FCUnpublish command
    FCUnpublish,
    /// Get stream length command
    GetStreamLength,
}

impl From<&str> for CommandType {
    fn from(s: &str) -> Self {
        match s {
            "connect" => CommandType::Connect,
            "call" => CommandType::Call,
            "createStream" => CommandType::CreateStream,
            "play" => CommandType::Play,
            "publish" => CommandType::Publish,
            "seek" => CommandType::Seek,
            "pause" => CommandType::Pause,
            "closeStream" => CommandType::CloseStream,
            "deleteStream" => CommandType::DeleteStream,
            "receiveAudio" => CommandType::ReceiveAudio,
            "receiveVideo" => CommandType::ReceiveVideo,
            "releaseStream" => CommandType::ReleaseStream,
            "FCPublish" => CommandType::FCPublish,
            "FCUnpublish" => CommandType::FCUnpublish,
            "getStreamLength" => CommandType::GetStreamLength,
            _ => CommandType::Connect, // Default
        }
    }
}

/// Protocol utilities
pub mod utils {
    use super::*;

    /// Calculate chunk header size
    pub fn chunk_header_size(format: &ChunkHeaderFormat) -> usize {
        match format {
            ChunkHeaderFormat::Format0 => 11,
            ChunkHeaderFormat::Format1 => 7,
            ChunkHeaderFormat::Format2 => 3,
            ChunkHeaderFormat::Format3 => 0,
        }
    }

    /// Check if extended timestamp is needed
    pub fn needs_extended_timestamp(timestamp: u32) -> bool {
        timestamp >= 0xFFFFFF
    }
}

/// Protocol constants
pub mod constants {
    /// Default chunk size
    pub const DEFAULT_CHUNK_SIZE: u32 = 128;

    /// Default window acknowledgement size
    pub const DEFAULT_WINDOW_ACK_SIZE: u32 = 2500000;

    /// Default peer bandwidth
    pub const DEFAULT_PEER_BANDWIDTH: u32 = 2500000;

    /// Bandwidth limit type: Hard
    pub const BANDWIDTH_LIMIT_HARD: u8 = 0;

    /// Bandwidth limit type: Soft
    pub const BANDWIDTH_LIMIT_SOFT: u8 = 1;

    /// Bandwidth limit type: Dynamic
    pub const BANDWIDTH_LIMIT_DYNAMIC: u8 = 2;
}