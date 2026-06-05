mod capture;
mod encoder;
mod recorder;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::Parser;
use recorder::Recorder;

#[derive(Parser)]
#[command(name = "screen-recorder")]
#[command(about = "Windows 录屏工具 — Rust 实现", long_about = None)]
#[command(version)]
struct Cli {
    /// 输出文件路径（支持 mp4/avi/mkv 等格式）
    #[arg(short, long, default_value = "output.mp4")]
    output: String,

    /// 目标帧率
    #[arg(short, long, default_value_t = 30)]
    fps: u32,

    /// 显示器索引（0 = 主显示器）
    #[arg(short, long, default_value_t = 0)]
    monitor: usize,

    /// 列出所有可用显示器
    #[arg(short, long)]
    list: bool,
}

fn main() {
    let cli = Cli::parse();

    // 列出显示器模式
    if cli.list {
        if let Err(e) = capture::ScreenCapture::list_monitors() {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // 检查 FFmpeg 是否可用
    if let Err(e) = encoder::check_ffmpeg() {
        eprintln!("错误: {}", e);
        std::process::exit(1);
    }

    // 初始化屏幕捕获
    let capture = match capture::ScreenCapture::new(cli.monitor) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("错误: {}", e);
            eprintln!("提示: 使用 --list 查看可用显示器");
            std::process::exit(1);
        }
    };

    // 录制状态标志（Ctrl+C 触发停止）
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\n⏹ 收到停止信号...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("设置 Ctrl+C 处理器失败");

    // 创建录制器并开始录制
    let recorder = Recorder::new(capture, cli.output, cli.fps, running);
    if let Err(e) = recorder.start() {
        eprintln!("录制错误: {}", e);
        std::process::exit(1);
    }
}
