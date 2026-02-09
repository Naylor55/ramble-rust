use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    // 配置监听地址
    let addr = "0.0.0.0:1935";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("正在监听 {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream).await {
                        eprintln!("处理客户端时出错: {:?}", e);
                    }
                });
            },
            Err(e) => {
                eprintln!("接受连接时出错: {:?}", e);
            }
        }
    }
}

async fn handle_client(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    println!("新客户端已连接");

    // RTMP 握手（简化版）
    let mut buffer = [0; 1536];
    // 使用 read_exact 确保完整读取 1536 字节握手数据
    stream.read_exact(&mut buffer).await?;

    // 发送 C1 和 S1
    let c1_s1: [u8; 1536] = [0; 1536];
    stream.write_all(&c1_s1).await?;

    // 读取 C2
    stream.read_exact(&mut buffer).await?;

    // 发送 S2
    let s2: [u8; 1536] = [0; 1536];
    stream.write_all(&s2).await?;

    // 处理 RTMP 消息
    loop {
        // 读完整的基本头(1) + 消息头(11) = 12 字节
        let mut header_buffer = [0; 12];
        stream.read_exact(&mut header_buffer).await?; // read_exact fills the buffer or returns an error

        // 解析消息头
        let chunk_type = header_buffer[0] & 0x3;
        // timestamp 和 body_length 是 3 字节（24-bit），手动组合为 u32
        let timestamp = ((header_buffer[1] as u32) << 16)
            | ((header_buffer[2] as u32) << 8)
            | (header_buffer[3] as u32);
        let body_length = ((header_buffer[4] as u32) << 16)
            | ((header_buffer[5] as u32) << 8)
            | (header_buffer[6] as u32);
        let message_type_id = header_buffer[7];
        // message_stream_id 是 4 字节，小端序
        let message_stream_id = u32::from_le_bytes([
            header_buffer[8],
            header_buffer[9],
            header_buffer[10],
            header_buffer[11],
        ]);

        println!(
            "接收到 RTMP 消息: chunk_type={}, timestamp={}, body_length={}, message_type_id={}, message_stream_id={}",
            chunk_type, timestamp, body_length, message_type_id, message_stream_id
        );

        // 读取消息体（read_exact 确保填满 buffer）
        let mut body_buffer = vec![0; body_length as usize];
        stream.read_exact(&mut body_buffer).await?;

        // 根据消息类型处理消息
        match message_type_id {
            0x01 => {
                // 设置块大小
                println!("设置块大小");
            }
            0x03 => {
                // 命令消息 AMF0
                println!("命令消息 AMF0");
            }
            0x04 => {
                // 数据消息 AMF0
                println!("数据消息 AMF0");
            }
            0x05 => {
                // 服务器控制消息
                println!("服务器控制消息");
            }
            0x06 => {
                // 用户控制消息
                println!("用户控制消息");
            }
            0x08 => {
                // 音频数据
                println!("音频数据");
            }
            0x09 => {
                // 视频数据
                println!("视频数据");
            }
            0x12 => {
                // 聚合消息
                println!("聚合消息");
            }
            _ => {
                println!("未知的消息类型 ID: {}", message_type_id);
            }
        }
    }
}
