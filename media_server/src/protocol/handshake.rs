
use bytes::{Bytes, BytesMut};
use rml_rtmp::handshake::{Handshake, HandshakeProcess};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::io::{self, Cursor};

pub struct RtmpHandshake {
    process: HandshakeProcess,
}

impl RtmpHandshake {
    pub fn new() -> Self {
        Self {
            process: Handshake::new().into_process(),
        }
    }

    pub async fn perform<T: AsyncReadExt + AsyncWriteExt + Unpin>(
        &mut self,
        stream: &mut T
    ) -> io::Result<()> {
        let mut buffer = BytesMut::with_capacity(1536 * 2);
        
        // 发送C0和C1
        let (c0, c1) = self.process.generate_client_handshake_bytes()?;
        stream.write_all(&c0).await?;
        stream.write_all(&c1).await?;
        
        // 读取S0、S1和S2
        buffer.resize(1536 * 2, 0);
        stream.read_exact(&mut buffer).await?;
        
        // 处理服务器握手数据
        let (s0, s1, s2) = (
            &buffer[0..1],
            &buffer[1..1537],
            &buffer[1537..3073],
        );
        
        self.process.process_server_bytes(s0, s1, Some(s2))?;
        
        // 发送C2
        let c2 = self.process.generate_client_response()?;
        stream.write_all(&c2).await?;
        
        Ok(())
    }
}