use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::capture::ScreenCapture;
use crate::encoder::VideoEncoder;

/// 录屏控制器，协调屏幕捕获与视频编码
pub struct Recorder {
    capture: ScreenCapture,
    output_path: String,
    fps: u32,
    running: Arc<AtomicBool>,
}

impl Recorder {
    pub fn new(capture: ScreenCapture, output_path: String, fps: u32, running: Arc<AtomicBool>) -> Self {
        Self {
            capture,
            output_path,
            fps,
            running,
        }
    }

    /// 开始录制，阻塞直到收到停止信号
    pub fn start(&self) -> Result<(), String> {
        let width = self.capture.width();
        let height = self.capture.height();

        println!("═══════════════════════════════════");
        println!("  录屏启动");
        println!("  分辨率: {}x{}", width, height);
        println!("  帧率:   {} fps", self.fps);
        println!("  输出:   {}", self.output_path);
        println!("  按 Ctrl+C 停止录制");
        println!("═══════════════════════════════════");

        let mut encoder = VideoEncoder::new(&self.output_path, width, height, self.fps)?;

        let frame_interval = Duration::from_secs_f64(1.0 / self.fps as f64);
        let mut frame_count: u64 = 0;
        let mut last_status = Instant::now();
        let start_time = Instant::now();

        while self.running.load(Ordering::SeqCst) {
            let frame_start = Instant::now();

            match self.capture.capture_frame() {
                Ok(image) => {
                    let data = image.as_raw();
                    if let Err(e) = encoder.write_frame(data) {
                        eprintln!("写入帧失败: {}", e);
                        break;
                    }
                    frame_count += 1;

                    // 每 3 秒打印一次状态
                    if last_status.elapsed() >= Duration::from_secs(3) {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let actual_fps = frame_count as f64 / elapsed;
                        println!(
                            "  录制中 | 帧: {} | 实际帧率: {:.1} fps | 时长: {:.1}s",
                            frame_count,
                            actual_fps,
                            elapsed
                        );
                        last_status = Instant::now();
                    }
                }
                Err(e) => {
                    eprintln!("截屏失败（跳过）: {}", e);
                }
            }

            // 控制帧率：不足一帧间隔则等待
            let elapsed = frame_start.elapsed();
            if elapsed < frame_interval {
                std::thread::sleep(frame_interval - elapsed);
            }
        }

        let total_duration = start_time.elapsed().as_secs_f64();
        println!("\n───────────────────────────────────");
        println!("  录制结束");
        println!("  总帧数: {}", frame_count);
        println!("  总时长: {:.1}s", total_duration);
        println!("  正在编码输出...");
        println!("───────────────────────────────────");

        encoder.finish()?;

        println!("✅ 录制完成！文件已保存: {}", self.output_path);

        Ok(())
    }
}
