use std::io::Write;
use std::process::{Child, Command, Stdio};

/// 视频编码模块，通过 FFmpeg 子进程将原始 RGBA 帧编码为 MP4
pub struct VideoEncoder {
    process: Child,
}

impl VideoEncoder {
    /// 创建编码器，启动 FFmpeg 子进程
    pub fn new(output_path: &str, width: u32, height: u32, fps: u32) -> Result<Self, String> {
        let process = Command::new("ffmpeg")
            .args([
                "-y",                          // 覆盖输出文件
                "-f", "rawvideo",              // 输入格式：原始视频
                "-pix_fmt", "rgba",            // 像素格式
                "-s", &format!("{}x{}", width, height), // 分辨率
                "-r", &fps.to_string(),        // 帧率
                "-i", "-",                     // 从 stdin 读取
                // 编码参数
                "-c:v", "libx264",             // H.264 编码
                "-preset", "ultrafast",        // 最快编码速度
                "-crf", "23",                  // 质量参数 (0-51, 越小越好)
                "-pix_fmt", "yuv420p",         // 输出像素格式（兼容性好）
                "-movflags", "+faststart",     // 允许边下边播
                output_path,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    "未找到 ffmpeg，请先安装 FFmpeg 并添加到系统 PATH\n  下载地址: https://ffmpeg.org/download.html".to_string()
                } else {
                    format!("启动 FFmpeg 失败: {}", e)
                }
            })?;

        Ok(Self { process })
    }

    /// 写入一帧 RGBA 数据
    pub fn write_frame(&mut self, frame_data: &[u8]) -> Result<(), String> {
        if let Some(stdin) = self.process.stdin.as_mut() {
            stdin
                .write_all(frame_data)
                .map_err(|e| format!("写入帧数据失败: {}", e))?;
            stdin
                .flush()
                .map_err(|e| format!("刷新缓冲区失败: {}", e))?;
        } else {
            return Err("FFmpeg stdin 已关闭".to_string());
        }
        Ok(())
    }

    /// 结束录制，等待 FFmpeg 完成编码
    pub fn finish(mut self) -> Result<(), String> {
        // 关闭 stdin 通知 FFmpeg 输入结束
        self.process.stdin.take();
        let status = self
            .process
            .wait()
            .map_err(|e| format!("等待 FFmpeg 结束失败: {}", e))?;
        if !status.success() {
            return Err(format!(
                "FFmpeg 编码异常退出，状态码: {:?}",
                status.code()
            ));
        }
        Ok(())
    }
}

/// 检查 FFmpeg 是否可用
pub fn check_ffmpeg() -> Result<(), String> {
    match Command::new("ffmpeg").arg("-version").output() {
        Ok(output) if output.status.success() => Ok(()),
        Ok(_) => Err("FFmpeg 运行异常".to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err("未找到 ffmpeg，请先安装 FFmpeg 并添加到系统 PATH\n  下载地址: https://ffmpeg.org/download.html".to_string())
        }
        Err(e) => Err(format!("检查 FFmpeg 失败: {}", e)),
    }
}
