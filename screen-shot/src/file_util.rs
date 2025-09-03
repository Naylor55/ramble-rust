use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// 读取 rtsp.txt，返回所有 rtsp url 的集合（去重）
pub fn read_rtsp_urls<P: AsRef<Path>>(file_path: P) -> io::Result<HashSet<String>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut urls = HashSet::new();
    for line in reader.lines() {
        if let Ok(l) = line {
            // 每行格式: camera_id,rtsp_url
            if let Some((_, rtsp)) = l.split_once(',') {
                let rtsp = rtsp.trim();
                if !rtsp.is_empty() {
                    urls.insert(rtsp.to_string());
                }
            }
        }
    }
    Ok(urls)
}

pub fn read_rtsp<P: AsRef<Path>>(file_path: P) -> io::Result<HashSet<String>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut urls = HashSet::new();
    for line in reader.lines() {
        if let Ok(l) = line {
            let rtsp = l.trim();
            if !rtsp.is_empty() {
                urls.insert(rtsp.to_string());
            }
        }
    }
    Ok(urls)
}
