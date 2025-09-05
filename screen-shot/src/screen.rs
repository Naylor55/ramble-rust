use crate::file_util;
use chrono::Local;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use threadpool::ThreadPool;

pub fn capture_rtsp_frame_simple(rtsp_url: &str, output_path: &Path) -> io::Result<()> {
    // 确保 img 目录存在
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let output_str = output_path.to_string_lossy().to_string();
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
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let err_msg = format!("ffmpeg failed (status: {}):\n{}", out.status, stderr);
                // 写入 logs/error.log
                let log_path = Path::new("logs").join("error.log");
                if let Some(parent) = log_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Ok(mut f) = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path)
                {
                    let _ = writeln!(f, "{}", err_msg);
                }
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "ffmpeg failed; see log",
                ))
            }
        }
        Err(e) => {
            let err_msg = format!("Failed to spawn ffmpeg: {}", e);
            // 写入 logs/error.log
            let log_path = Path::new("logs").join("error.log");
            if let Some(parent) = log_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(mut f) = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
            {
                let _ = writeln!(f, "{}", err_msg);
            }
            Err(e)
        }
    }
}

///截图
pub fn screen_shot() {
    // 调用 file_util::read_rtsp_urls 读取 rtsp.txt 并对每个 rtsp url 截图
    let rtsp_set = file_util::read_rtsp("rtsp.txt");
    print!("Read rtsp_set: {:?}", rtsp_set);
    let start = Instant::now();
    match rtsp_set {
        Ok(urls) => {
            for (idx, rtsp) in urls.iter().enumerate() {
                let out_path = format!("img/rtsp_{}.jpg", idx + 1);
                match capture_rtsp_frame_simple(rtsp, std::path::Path::new(&out_path)) {
                    Ok(_) => println!("Saved screenshot for {} -> {}", rtsp, out_path),
                    Err(e) => eprintln!("Failed to capture for {}: {}", rtsp, e),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read rtsp.txt: {}", e);
        }
    }
    let duration = start.elapsed();
    println!("执行耗时: {:?}", duration);
}

/// 多线程并发截图
pub fn do_shot_with_thread(rtsp_file: &str) {
    let start = std::time::Instant::now();
    let rtsp_set = file_util::read_rtsp(rtsp_file);
    match rtsp_set {
        Ok(urls) => {
            let mut handles = vec![];
            for (idx, rtsp) in urls.iter().enumerate() {
                let rtsp = rtsp.clone();
                let out_path = format!("img/rtsp_{}.jpg", idx + 1);

                // 每个任务开一个线程
                let handle = thread::spawn(move || {
                    let output_str = out_path.clone();
                    let mut cmd = Command::new("ffmpeg");
                    cmd.arg("-y")
                        .arg("-rtsp_transport")
                        .arg("tcp")
                        .arg("-i")
                        .arg(&rtsp)
                        .arg("-vframes")
                        .arg("1")
                        .arg("-q:v")
                        .arg("2")
                        .arg(&output_str)
                        .stderr(Stdio::piped())
                        .stdout(Stdio::null());

                    match cmd.output() {
                        Ok(out) => {
                            if out.status.success() {
                                println!("Saved screenshot for {} -> {}", rtsp, output_str);
                            } else {
                                let stderr = String::from_utf8_lossy(&out.stderr);
                                let err_msg =
                                    format!("ffmpeg failed (status: {}):\n{}", out.status, stderr);
                                log_error(&err_msg);
                                eprintln!("Failed to capture for {}: {}", rtsp, err_msg);
                            }
                        }
                        Err(e) => {
                            let err_msg = format!("Failed to spawn ffmpeg: {}", e);
                            log_error(&err_msg);
                            eprintln!("Failed to capture for {}: {}", rtsp, e);
                        }
                    }
                });
                handles.push(handle);
            }

            // 等待所有线程结束
            for h in handles {
                let _ = h.join();
            }
        }
        Err(e) => {
            eprintln!("Failed to read {}: {}", rtsp_file, e);
        }
    }
    println!("并行截图耗时: {:?}", start.elapsed());
}

/// 错误日志写入 logs/error.log
fn log_error(err_msg: &str) {
    let log_path = Path::new("logs").join("error.log");
    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(f, "{}", err_msg);
    }
}

/// 使用线程池控制并发数量
pub fn do_shot_with_threadpool(rtsp_file: &str) {
    let start = std::time::Instant::now();
    let rtsp_set = file_util::read_rtsp(rtsp_file);
    match rtsp_set {
        Ok(urls) => {
            //设置线程池大小
            let pool = ThreadPool::new(5);
            let urls = Arc::new(urls);

            for (idx, rtsp) in urls.iter().enumerate() {
                let rtsp = rtsp.clone();
                let now = Local::now();
                let ts = now.format("%y%m%d_%H%M%S").to_string(); // 250909_120101
                let out_path = format!("img/rtsp_{}_{}.jpg", idx + 1, ts);

                pool.execute(move || {
                    let output_str = out_path.clone();
                    let mut cmd = Command::new("ffmpeg");
                    cmd.arg("-y")
                        .arg("-rtsp_transport")
                        .arg("tcp")
                        .arg("-i")
                        .arg(&rtsp)
                        .arg("-vframes")
                        .arg("1")
                        .arg("-q:v")
                        .arg("2")
                        .arg(&output_str)
                        .stderr(Stdio::piped())
                        .stdout(Stdio::null());

                    match cmd.output() {
                        Ok(out) => {
                            if out.status.success() {
                                println!("Saved screenshot for {} -> {}", rtsp, output_str);
                            } else {
                                let stderr = String::from_utf8_lossy(&out.stderr);
                                let err_msg =
                                    format!("ffmpeg failed (status: {}):\n{}", out.status, stderr);
                                log_error(&err_msg);
                                eprintln!("Failed to capture for {}: {}", rtsp, err_msg);
                            }
                        }
                        Err(e) => {
                            let err_msg = format!("Failed to spawn ffmpeg: {}", e);
                            log_error(&err_msg);
                            eprintln!("Failed to capture for {}: {}", rtsp, e);
                        }
                    }
                });
            }

            // 等待线程池所有任务完成
            pool.join();
        }
        Err(e) => {
            eprintln!("Failed to read {}: {}", rtsp_file, e);
        }
    }
    println!("并行截图耗时: {:?}", start.elapsed());
}
