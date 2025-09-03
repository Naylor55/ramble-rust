
use bytes::Bytes;
use rml_rtmp::chunk::{ChunkHeader, ChunkStreamReader, ChunkStreamWriter};
use rml_rtmp::session::StreamMetadata;
use std::collections::HashMap;

pub struct ChunkProcessor {
    reader: ChunkStreamReader,
    writer: ChunkStreamWriter,
    chunk_size: u32,
    stream_metadata: HashMap<u32, StreamMetadata>,
}

impl ChunkProcessor {
    pub fn new() -> Self {
        Self {
            reader: ChunkStreamReader::new(1536), // 默认chunk大小
            writer: ChunkStreamWriter::new(1536),
            chunk_size: 1536,
            stream_metadata: HashMap::new(),
        }
    }

    pub fn set_chunk_size(&mut self, new_size: u32) {
        self.chunk_size = new_size;
        self.reader.set_max_chunk_size(new_size);
        self.writer.set_chunk_size(new_size);
    }

    pub fn process_received_data(&mut self, data: &[u8]) -> Vec<(ChunkHeader, Bytes)> {
        let mut chunks = Vec::new();
        let mut cursor = std::io::Cursor::new(data);
        
        while let Ok(Some((header, payload))) = self.reader.read_chunk(&mut cursor) {
            chunks.push((header, payload));
        }
        
        chunks
    }
}