mod streamhub;

// 后续代码可以使用 streamhub 模块中的内容
use streamhub::define::{FrameData, StreamHubEvent};

use axum::{
    routing::get,
    Router,
    http::HeaderValue,
    body::StreamBody,
    extract::Path,
    response::IntoResponse,
};
use log::info;
use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};
use streamhub::define::{FrameData, StreamHubEvent, StreamHubEventSender};

// 模拟 StreamHub
struct StreamHub {
    event_sender: StreamHubEventSender,
}

impl StreamHub {
    fn new() -> Self {
        let (event_sender, _) = mpsc::unbounded_channel();
        StreamHub { event_sender }
    }

    fn get_event_sender(&self) -> StreamHubEventSender {
        self.event_sender.clone()
    }
}

// RTMP 服务器
async fn start_rtmp_server(event_sender: StreamHubEventSender) {
    // 模拟 RTMP 服务器启动
    info!("RTMP server started");
    // 这里应该处理 RTMP 连接和流数据
    // 并将流数据通过 event_sender 发送到 StreamHub
}

// HTTP-FLV 服务器
async fn http_flv_handler(
    Path(stream_name): Path<String>,
    event_sender: StreamHubEventSender,
) -> impl IntoResponse {
    let (result_sender, result_receiver) = oneshot::channel();
    let event = StreamHubEvent::Subscribe {
        identifier: stream_name.clone(),
        result_sender,
    };
    event_sender.send(event).unwrap();

    let receiver = result_receiver.await.unwrap().unwrap();

    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(receiver)
       .map(|frame| match frame {
            FrameData::Video { data, .. } | FrameData::Audio { data, .. } | FrameData::MetaData { data, .. } => data.freeze(),
        });

    let body = StreamBody::new(stream);
    let mut response = body.into_response();
    response.headers_mut().insert("Content-Type", HeaderValue::from_static("video/x-flv"));
    response
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stream_hub = StreamHub::new();
    let event_sender = stream_hub.get_event_sender();

    // 启动 RTMP 服务器
    tokio::spawn(start_rtmp_server(event_sender.clone()));

    // 启动 HTTP-FLV 服务器
    let app = Router::new()
       .route("/flv/:stream_name", get(http_flv_handler))
       .with_state(event_sender);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("HTTP-FLV server listening on {}", addr);
    axum::Server::bind(&addr)
       .serve(app.into_make_service())
       .await
       .unwrap();
}