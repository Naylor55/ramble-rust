// ======================== 回退：恢复两步前的单参数截图实现（已注释保留） ========================
// 原先实现：接受一个 rtsp URL 参数，直接截图保存为 ./screenshot.jpg
// 便于回退，保留为注释。
//
// use std::env;
// use std::fs::{self, File};
// use std::io::{self, Write};
// use std::path::Path;
// use std::process::{Command, Stdio};
// use std::time::{SystemTime, UNIX_EPOCH};
//
// fn log_error(log_path: &Path, msg: &str) -> io::Result<()> {
//     if let Some(parent) = log_path.parent() {
//         fs::create_dir_all(parent)?;
//     }
//     let mut f = File::options().create(true).append(true).open(log_path)?;
//     let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
//     writeln!(f, "[{}] {}", now, msg)?;
//     Ok(())
// }
//
// fn capture_rtsp_frame(rtsp_url: &str, output: &Path, log_path: &Path) -> io::Result<()> {
//     let output_str = output.to_string_lossy().to_string();
//     let mut cmd = Command::new("ffmpeg");
//     cmd.arg("-y")
//         .arg("-rtsp_transport").arg("tcp")
//         .arg("-i").arg(rtsp_url)
//         .arg("-vframes").arg("1")
//         .arg("-q:v").arg("2")
//         .arg(&output_str)
//         .stderr(Stdio::piped())
//         .stdout(Stdio::null());
//
//     let output_res = cmd.output();
//     match output_res {
//         Ok(out) => {
//             if out.status.success() {
//                 return Ok(());
//             } else {
//                 let stderr = String::from_utf8_lossy(&out.stderr);
//                 let err_msg = format!("ffmpeg failed (status: {}):\n{}", out.status, stderr);
//                 let _ = log_error(log_path, &err_msg);
//                 return Err(io::Error::new(io::ErrorKind::Other, "ffmpeg failed; see log"));
//             }
//         }
//         Err(e) => {
//             let err_msg = format!("Failed to spawn ffmpeg: {}", e);
//             let _ = log_error(log_path, &err_msg);
//             return Err(e);
//         }
//     }
// }
//
// fn main() {
//     let args: Vec<String> = env::args().collect();
//     if args.len() != 2 {
//         eprintln!("Usage: {} <rtsp_url>", args.get(0).unwrap_or(&"screen-shot".to_string()));
//         std::process::exit(1);
//     }
//     let rtsp = &args[1];
//     let out = std::path::Path::new("./screenshot.jpg");
//     let log_path = std::path::Path::new("logs").join("error.log");
//     if let Err(e) = capture_rtsp_frame(rtsp, out, &log_path) {
//         eprintln!("Error: {}. See log: {}", e, log_path.display());
//         std::process::exit(2);
//     }
// }
// ======================== 注释区结束 ========================

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod file_util;
mod live;
mod screen;

fn log_error(log_path: &Path, msg: &str) -> io::Result<()> {
    // ensure logs dir exists
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = File::options().create(true).append(true).open(log_path)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    writeln!(f, "[{}] {}", now, msg)?;
    Ok(())
}

fn capture_rtsp_frame(rtsp_url: &str, output: &Path, log_path: &Path) -> io::Result<()> {
    // Build ffmpeg command:
    // ffmpeg -y -rtsp_transport tcp -i "<rtsp_url>" -vframes 1 -q:v 2 <output>
    let output_str = output.to_string_lossy().to_string();
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-rtsp_transport")
        .arg("tcp")
        .arg("-i")
        .arg(rtsp_url)
        .arg("-vframes")
        .arg("1")
        .arg("-q:v")
        .arg("2")
        .arg(&output_str)
        .stderr(Stdio::piped())
        .stdout(Stdio::null());

    let output_res = cmd.output();
    match output_res {
        Ok(out) => {
            if out.status.success() {
                return Ok(());
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let err_msg = format!("ffmpeg failed (status: {}):\n{}", out.status, stderr);
                let _ = log_error(log_path, &err_msg);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "ffmpeg failed; see log",
                ));
            }
        }
        Err(e) => {
            let err_msg = format!("Failed to spawn ffmpeg: {}", e);
            let _ = log_error(log_path, &err_msg);
            return Err(e);
        }
    }
}

fn main() {
    //根据cameraIds 获取rtsp串并写入 rtsp.txt
    // live::find_rtsp_url();

    //单线程并行截图
    // screen::screen_shot();

    //多线程方案
    // screen::do_shot_with_thread("rtsp.txt");

    //线程池方案
    screen::do_shot_with_threadpool("rtsp.txt");

    //opencv + 线程池方案
    // screen.do_shot_with_opencv("rtsp.txt")
}
