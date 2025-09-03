use crate::file_util;
use chrono::Local;
use opencv::{core, imgcodecs, prelude::*, videoio};
use std::fs;
use std::io;
use std::io::Write;
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

///截图 - 使用opencv 并且是多线程
/// 需要系统安装 OpenCV（动态库），Rust crate 只是绑定
/// 内部通过 OpenCV 的 C++ API 拉流、解码；在 Rust 进程内执行，不启动新进程
/// 截图速度快，不启动子进程，CPU/内存占用更低，可按需抓帧；线程池可控制并发
/// 可以直接拿到 Mat 做进一步处理（缩放、画标注等）
pub fn do_shot_with_opencv(rtsp_file: &str) {
    let rtsp_set = file_util::read_rtsp(rtsp_file);

    match rtsp_set {
        Ok(urls) => {
            let pool_size = 4; // 可以根据需要调整
            let pool = ThreadPool::new(pool_size);
            let urls = Arc::new(urls);

            for (idx, url) in urls.iter().enumerate() {
                let url = url.clone();
                pool.execute(move || match capture_rtsp_frame(&url, idx) {
                    Ok(path) => println!("Saved: {}", path),
                    Err(e) => {
                        let err_msg = format!("Failed {}: {}", url, e);
                        log_error(&err_msg);
                        eprintln!("{}", err_msg);
                    }
                });
            }

            pool.join(); // 等待所有任务完成
        }
        Err(e) => {
            eprintln!("Failed to read {}: {}", rtsp_file, e);
        }
    }
}

/// 截图一帧并保存
fn capture_frame_with_opencv(rtsp_url: &str, idx: usize) -> opencv::Result<String> {
    let mut cap = videoio::VideoCapture::from_file(rtsp_url, videoio::CAP_FFMPEG)?;
    if !videoio::VideoCapture::is_opened(&cap)? {
        return Err(opencv::Error::new(
            core::StsError,
            "Failed to open RTSP".to_string(),
        ));
    }

    let mut frame = Mat::default();

    // 尝试读取一帧
    for _ in 0..5 {
        cap.read(&mut frame)?;
        if !frame.empty() {
            break;
        }
    }

    if frame.empty() {
        return Err(opencv::Error::new(
            core::StsError,
            "No frame captured".to_string(),
        ));
    }

    // 时间戳命名
    let ts = Local::now().format("%y%m%d_%H%M%S").to_string();
    let out_path = format!("img/rtsp_{}_{}.jpg", idx + 1, ts);

    // 确保目录存在
    if let Some(parent) = Path::new(&out_path).parent() {
        let _ = fs::create_dir_all(parent);
    }

    // 保存图片
    imgcodecs::imwrite(&out_path, &frame, &opencv::types::VectorOfi32::new())?;

    Ok(out_path)
}
