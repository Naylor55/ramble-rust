//! 简单的 RTMP/HTTP-FLV 辅助服务（占位/调试用途）
//!
//! 功能概述：
//! - 在 TCP 1935 上提供一个占位的 RTMP TCP 监听（仅保持连接，并不实现完整 RTMP 协议）
//! - 在 HTTP 8080 上提供 HTTP-FLV 发布/订阅端点：POST /live/{stream} 发布原始 FLV 数据（服务器会过滤音频 tag），GET /live/{stream} 拉流
//! - 对每个流保存 FLV header 与 AVC sequence header（若有），在新订阅者连接时先发送它们，保证 ffplay 能够解析 H.264 视频流
//! - 日志中提供十六进制预览，便于调试头信息与 tag
use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info, warn, debug};
use tracing_subscriber;

use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use async_stream::stream;
use bytes::Bytes;
use std::collections::HashMap;
use futures::StreamExt; // 用于 Body.next().await

/// 程序命令行参数定义
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address to bind the server to (默认 0.0.0.0:1935)
    #[arg(short, long, default_value = "0.0.0.0:1935")]
    address: String,

    /// Log level (error/warn/info/debug/trace)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

// StreamState 保存每个流的广播通道以及可选的头信息（FLV header + AVC sequence）
struct StreamState {
    sender: broadcast::Sender<Bytes>,
    header: Option<Bytes>,   // FLV header (9) + PrevTagSize0 (4)
    avc_seq: Option<Bytes>,  // AVC sequence header tag（完整 tag bytes，包括尾部 prev size）
}
type Streams = Arc<Mutex<HashMap<String, StreamState>>>;

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let args = Args::parse();

    // 初始化日志
    init_logging(&args.log_level)?;

    info!("Starting RTMP Streaming Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Server will bind to: {}", args.address);

    // 解析监听地址（用于 TCP RTMP placeholder）
    let addr: SocketAddr = args.address.parse()?;

    // 创建流注册表并启动 HTTP-FLV 服务（后台任务）
    let streams: Streams = Arc::new(Mutex::new(HashMap::new()));
    let http_streams = streams.clone();

    tokio::spawn(async move {
        if let Err(e) = run_http_flv_server(http_streams).await {
            error!("HTTP-FLV server error: {}", e);
        }
    });

    // 运行 TCP（RTMP 占位）服务，保持向 ffmpeg 的 TCP 连接可用
    if let Err(e) = run_server(addr).await {
        error!("Server error: {}", e);
        return Err(e);
    }

    Ok(())
}

/// 初始化日志（简短中文说明）
/// level: "error" / "warn" / "info" / "debug" / "trace"
fn init_logging(level: &str) -> Result<()> {
    let log_level = match level.to_lowercase().as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        "trace" => tracing::Level::TRACE,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    Ok(())
}

/// 在给定地址上监听 TCP（占位 RTMP），每个连接读入并丢弃数据（保持连接活跃）
/// 说明：这里不实现 RTMP 握手/消息解析，仅作为占位，避免 ffmpeg 连接被拒绝。
async fn run_server(addr: SocketAddr) -> Result<()> {
    info!("Server listening on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    loop {
        let (socket, peer) = match listener.accept().await {
            Ok(p) => p,
            Err(e) => {
                warn!("Accept failed: {}", e);
                continue;
            }
        };
        info!("Accepted connection from {}", peer);
        tokio::spawn(async move {
            if let Err(e) = handle_socket(socket, peer).await {
                warn!("Connection {} closed with error: {}", peer, e);
            } else {
                info!("Connection {} closed", peer);
            }
        });
    }
}

/// 启动 HTTP-FLV 服务：监听 0.0.0.0:8080，提供 /live/{stream} 的 GET/POST
async fn run_http_flv_server(streams: Streams) -> Result<()> {
    let make_svc = make_service_fn(move |_| {
        let streams = streams.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handle_http(req, streams.clone())
            }))
        }
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    info!("Starting HTTP-FLV server on {}", addr);
    let server = Server::bind(&addr).serve(make_svc);
    server.await.map_err(|e: hyper::Error| anyhow::anyhow!(e))?;
    Ok(())
}

/// HTTP 请求处理
/// - POST /live/{stream} 作为 publisher：接收 FLV 字节流（按 tag 解析），丢弃 audio tag，仅转发 video/script tag
/// - GET  /live/{stream} 作为 subscriber：先发送已保存的 FLV header 与 AVC seq header（若有），再转发广播的字节
async fn handle_http(req: Request<Body>, streams: Streams) -> Result<Response<Body>, hyper::Error> {
    // 验证路径 /live/{stream}
    let path = req.uri().path().to_string();
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() != 3 || parts[1] != "live" {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap());
    }
    let stream_name = parts[2].to_string();

    match (req.method(), req.uri().path()) {
        // Publisher: POST /live/{stream}
        (&Method::POST, _) => {
            // 确保流状态存在（创建或复用），并获得广播 sender
            let sender = {
                let mut map = streams.lock().await;
                if map.contains_key(&stream_name) {
                    debug!("Publisher connecting to existing stream '{}'", stream_name);
                } else {
                    let (tx, _rx) = broadcast::channel::<Bytes>(1024);
                    map.insert(stream_name.clone(), StreamState { sender: tx.clone(), header: None, avc_seq: None });
                    info!("Created stream state for '{}'", stream_name);
                }
                map.get(&stream_name).unwrap().sender.clone()
            };
 
            // 以流式方式读取请求体的块并缓冲，按 FLV tag 解析后转发或过滤
            let mut body = req.into_body();
            let mut buf: Vec<u8> = Vec::new();
            let mut header_stored = false;

            while let Some(chunk) = body.next().await {
                match chunk {
                    Ok(bytes) => {
                        buf.extend_from_slice(&bytes);

                        // 先获取并处理 FLV header（9 字节 header + 4 字节 PrevTagSize0）
                        if !header_stored {
                            if buf.len() >= 13 && &buf[..3] == b"FLV" {
                                // 读取原始 header，记录日志并根据实际转发情况调整 flags（去掉 audio 位，若检测到 video 则设置 video 位）
                                let original_header = buf[..13].to_vec();
                                let mut header_bytes = original_header.clone();

                                // 日志：原始 header 预览与 flags
                                let orig_preview = hex_preview(&original_header, 16);
                                let orig_flags = original_header[4];
                                info!("Publisher [{}]: received FLV header (orig): {} flags=0x{:02x} ({})", stream_name, orig_preview, orig_flags, flag_bits(orig_flags));

                                // 扫描缓冲区判断后面是否有 video tag（tag_type == 9）
                                let mut has_video = false;
                                let mut idx: usize = 13;
                                while idx + 11 <= buf.len() {
                                    let tt = buf[idx];
                                    if tt == 9 {
                                        has_video = true;
                                        break;
                                    }
                                    let data_size = ((buf[idx + 1] as usize) << 16)
                                        | ((buf[idx + 2] as usize) << 8)
                                        | (buf[idx + 3] as usize);
                                    let adv = 11usize + data_size + 4usize;
                                    if idx + adv > buf.len() {
                                        break;
                                    }
                                    idx += adv;
                                }

                                // 因为服务器会丢弃 audio tag，所以清除 header 中的 audio 位（0x01）。
                                // 如果检测到 video tag，则设置 video 位（0x04）。
                                header_bytes[4] &= !0x01u8;
                                if has_video {
                                    header_bytes[4] |= 0x04u8;
                                }

                                // 日志：修改后的 header 预览与 flags
                                let mod_preview = hex_preview(&header_bytes, 16);
                                let mod_flags = header_bytes[4];
                                info!("Publisher [{}]: modified FLV header (mod): {} flags=0x{:02x} ({})", stream_name, mod_preview, mod_flags, flag_bits(mod_flags));

                                // 将修改后的 header 保存到 StreamState，并立即转发给当前订阅者
                                {
                                    let mut map = streams.lock().await;
                                    if let Some(s) = map.get_mut(&stream_name) {
                                        s.header = Some(Bytes::from(header_bytes.clone()));
                                    }
                                }
                                let _ = sender.send(Bytes::from(header_bytes.clone()));
                                // 消耗缓冲区中的 header bytes
                                buf.drain(..13);
                                header_stored = true;
                            } else {
                                // header 未接收完整，等待更多数据
                                continue;
                            }
                        }

                        // 开始解析后续 tags：每个 tag = 11-byte header + data_size + 4-byte PreviousTag
                        loop {
                            if buf.len() < 11 {
                                break;
                            }
                            let tag_type = buf[0];
                            let data_size = ((buf[1] as usize) << 16) | ((buf[2] as usize) << 8) | (buf[3] as usize);
                            let total_tag_len = 11usize + data_size + 4;
                            if buf.len() < total_tag_len {
                                break; // 等待更多数据
                            }

                            if tag_type == 8 {
                                // 音频 tag：直接丢弃
                                buf.drain(..total_tag_len);
                                continue;
                            } else {
                                // 取出完整 tag（含 header 和 trailing prev size）
                                let send_bytes_vec = buf.drain(..total_tag_len).collect::<Vec<u8>>();
                                // 若为 video tag，检查是否为 AVC sequence header（CodecID == 7 且 AVCPacketType == 0）
                                if tag_type == 9 {
                                    // payload 起始于偏移 11
                                    if total_tag_len >= 11 + 2 {
                                        let payload0 = send_bytes_vec[11]; // FrameType/CodecID
                                        let codec_id = payload0 & 0x0f;
                                        if codec_id == 7 {
                                            // 需要至少一个字节 AVCPacketType
                                            if 11 + 1 < send_bytes_vec.len() {
                                                let avc_packet_type = send_bytes_vec[11 + 1];
                                                if avc_packet_type == 0 {
                                                    // AVC sequence header：保存并广播（用于新订阅者）
                                                    let seq_bytes = Bytes::from(send_bytes_vec.clone());
                                                    let seq_preview = hex_preview(&seq_bytes, 32);
                                                    info!("Publisher [{}]: detected AVC sequence header ({} bytes), preview: {}", stream_name, seq_bytes.len(), seq_preview);
                                                    let mut map = streams.lock().await;
                                                    if let Some(s) = map.get_mut(&stream_name) {
                                                        s.avc_seq = Some(seq_bytes.clone());
                                                    }
                                                    let _ = sender.send(seq_bytes);
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                }
                                // 非 AVC-seq 的普通 tag：转发（视频或 script）
                                debug!("Publisher [{}]: forwarding tag type={} data_size={}", stream_name, tag_type, data_size);
                                let _ = sender.send(Bytes::from(send_bytes_vec));
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Error reading publisher body for {}: {}", stream_name, e);
                        break;
                    }
                }
            }

            // 处理剩余缓冲区中的完整 tag（尽力转发或检测 avc seq）
            if header_stored {
                loop {
                    if buf.len() < 11 {
                        break;
                    }
                    let tag_type = buf[0];
                    let data_size = ((buf[1] as usize) << 16) | ((buf[2] as usize) << 8) | (buf[3] as usize);
                    let total_tag_len = 11usize + data_size + 4;
                    if buf.len() < total_tag_len {
                        break;
                    }
                    if tag_type == 8 {
                        buf.drain(..total_tag_len);
                        continue;
                    } else {
                        let send_bytes_vec = buf.drain(..total_tag_len).collect::<Vec<u8>>();
                        if tag_type == 9 {
                            if total_tag_len >= 11 + 2 {
                                let payload0 = send_bytes_vec[11];
                                let codec_id = payload0 & 0x0f;
                                if codec_id == 7 {
                                    if 11 + 1 < send_bytes_vec.len() {
                                        let avc_packet_type = send_bytes_vec[11 + 1];
                                        if avc_packet_type == 0 {
                                            let seq_bytes = Bytes::from(send_bytes_vec.clone());
                                            let seq_preview = hex_preview(&seq_bytes, 32);
                                            info!("Publisher [{}]: detected AVC sequence header (leftover) ({} bytes), preview: {}", stream_name, seq_bytes.len(), seq_preview);
                                            let mut map = streams.lock().await;
                                            if let Some(s) = map.get_mut(&stream_name) {
                                                s.avc_seq = Some(seq_bytes.clone());
                                            }
                                            let _ = sender.send(seq_bytes);
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                        // 转发剩余的非音频 tag
                        debug!("Publisher [{}]: forwarding leftover tag type={} data_size={}", stream_name, tag_type, data_size);
                        let _ = sender.send(Bytes::from(send_bytes_vec));
                        continue;
                    }
                }
            }

            // 发布者断开或 EOF：移除该流状态，避免占用资源
            {
                let mut map = streams.lock().await;
                if map.remove(&stream_name).is_some() {
                    info!("Publisher for {} disconnected; removed stream state", stream_name);
                }
            }

            Ok(Response::new(Body::from("OK")))
         }

        // Subscriber: GET /live/{stream}
        (&Method::GET, _) => {
            // 确保 StreamState 存在并获取 header/avc_seq/sender（不存在则新建，便于后续 publisher 连接）
            let (maybe_header, maybe_avc_seq, sender) = {
                let mut map = streams.lock().await;
                if !map.contains_key(&stream_name) {
                    let (tx, _rx) = broadcast::channel::<Bytes>(1024);
                    map.insert(stream_name.clone(), StreamState { sender: tx.clone(), header: None, avc_seq: None });
                    info!("Created stream state for subscriber '{}'", stream_name);
                } else {
                    debug!("Subscriber connecting to existing stream '{}'", stream_name);
                }
                let state = map.get(&stream_name).unwrap();
                (state.header.clone(), state.avc_seq.clone(), state.sender.clone())
            };

            // 为该订阅者创建接收器，并在连接时记录日志（是否存在 header / avc_seq）
            let mut rx = sender.subscribe();
            let sub_count = sender.receiver_count();
            info!("Subscriber connected to '{}' (subscribers: {}) header={}, avc_seq={}", stream_name, sub_count, maybe_header.is_some(), maybe_avc_seq.is_some());

            // 响应流：若存在 header/avc_seq 先发送，再发送后续广播数据
            let body_stream = stream! {
                if let Some(h) = maybe_header {
                    yield Ok::<Bytes, std::convert::Infallible>(h);
                }
                if let Some(seq) = maybe_avc_seq {
                    yield Ok::<Bytes, std::convert::Infallible>(seq);
                }
                loop {
                    match rx.recv().await {
                        Ok(b) => {
                            yield Ok::<Bytes, std::convert::Infallible>(b);
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            // 若订阅者落后，丢弃一些包继续
                            continue;
                        }
                        Err(_) => {
                            // 发送端关闭，结束流
                            break;
                        }
                    }
                }
            };

            let response = Response::builder()
                .header("Content-Type", "video/x-flv")
                .status(StatusCode::OK)
                .body(Body::wrap_stream(body_stream))
                .unwrap();
            Ok(response)
        }

        _ => Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Body::from("Method Not Allowed"))
            .unwrap()),
    }
}

/// 处理 TCP socket：将读取到的数据丢弃（占位用）
async fn handle_socket(mut socket: TcpStream, peer: SocketAddr) -> Result<()> {
    // 只做 copy 到 sink，保持连接打开
    let mut sink = io::sink();
    let n = io::copy(&mut socket, &mut sink).await?;
    info!("Read {} bytes from {}", n, peer);
    Ok(())
}

// -------------------- 辅助日志函数（被使用，不删除） --------------------

/// 返回前 max 个字节的十六进制预览字符串（用于日志）
fn hex_preview(data: &[u8], max: usize) -> String {
    data.iter()
        .take(max)
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// 将 FLV flags 字节（第 5 字节）转换为可读的位表示（A 音频，R 保留，V 视频）
fn flag_bits(b: u8) -> String {
    let audio = if b & 0x01 != 0 { "A" } else { "-" };
    let reserved = if b & 0x02 != 0 { "R" } else { "-" };
    let video = if b & 0x04 != 0 { "V" } else { "-" };
    format!("{}{}{}", audio, reserved, video)
}