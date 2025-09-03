
use openh264::decoder::Decoder;
use scuffle_h264::AvcDecoderConfigurationRecord;
use bytes::Bytes;

pub struct H264Processor {
    decoder: Decoder,
    config: Option<AvcDecoderConfigurationRecord>,
}

impl H264Processor {
    pub fn new() -> Self {
        Self {
            decoder: Decoder::new().expect("Failed to create H264 decoder"),
            config: None,
        }
    }

    pub fn set_config(&mut self, config_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.config = Some(AvcDecoderConfigurationRecord::parse(&mut std::io::Cursor::new(config_data))?);
        Ok(())
    }

    pub fn process_nalu(&mut self, nalu_data: &[u8]) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        // 处理单个NAL单元
        match self.decoder.decode(nalu_data) {
            Ok(Some(yuv_data)) => Ok(Some(yuv_data)),
            Ok(None) => Ok(None), // 需要更多数据
            Err(e) => Err(Box::new(e)),
        }
    }
}