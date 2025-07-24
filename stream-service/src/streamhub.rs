pub mod define {
    use bytes::BytesMut;
    use serde::{Deserialize, Serialize};
    use tokio::sync::{mpsc, oneshot};

    #[derive(Clone, Serialize, Deserialize)]
    pub enum FrameData {
        Video { timestamp: u32, data: BytesMut },
        Audio { timestamp: u32, data: BytesMut },
        MetaData { timestamp: u32, data: BytesMut },
    }

    pub type FrameDataSender = mpsc::UnboundedSender<FrameData>;
    pub type FrameDataReceiver = mpsc::UnboundedReceiver<FrameData>;

    pub type StreamHubEventSender = mpsc::UnboundedSender<StreamHubEvent>;

    #[derive(Serialize, Deserialize)]
    pub enum StreamHubEvent {
        Subscribe {
            identifier: String,
            result_sender: oneshot::Sender<Result<FrameDataReceiver, ()>>,
        },
        Publish {
            identifier: String,
            data: FrameDataSender,
        },
    }
}