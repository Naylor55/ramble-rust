use serde_json;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::time::Duration;

pub fn find_rtsp_url() {
    let camera_ids = vec![
        "49325491131253683",
        "49325491131253672",
        "49325491131253724",
        "49325491131253684",
        "49325491131253726",
        "49325491131253678",
        "49325491131253729",
        "49325491131253686",
        "49325491131253704",
        "49325491131253685",
        "49325491131253687",
        "49325491131253703",
        "49325514384474129",
        "49325491131253679",
        "49325514384474130",
    ];
    //先清空 rtsp.txt
    clear_file("rtsp.txt");

    println!("rtsp.txt文件已清空");
    for cam in &camera_ids {
        match get_live_url(cam) {
            Ok(rtsp) => {
                println!("{} -> {}", cam, rtsp);
            }
            Err(e) => {
                eprintln!("failed to get rtsp for {}: {}", cam, e);
            }
        }
    }
}

///清空rtsp.txt
fn clear_file(path: &str) {
    if let Err(e) = OpenOptions::new().write(true).truncate(true).open(path) {
        eprintln!("清空文件失败: {}, 错误: {}", path, e);
        // 仅打印错误，不返回，让程序继续执行
    }
}

///获取rtsp串，根据相机id
// /// 调用的是127的地址
// pub fn get_rtsp_url(camera_id: &str) -> Result<String, io::Error> {
//     // 构建请求 URL
//     let url = format!(
//         "http://127.0.0.1:8083/report/rtspUrl?cameraIndexCode={}",
//         camera_id
//     );

//     let client = reqwest::blocking::Client::builder()
//         .timeout(Duration::from_secs(5))
//         .build()
//         .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

//     let resp = client
//         .get(&url)
//         .send()
//         .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

//     if !resp.status().is_success() {
//         return Err(io::Error::new(
//             io::ErrorKind::Other,
//             format!("HTTP status: {}", resp.status()),
//         ));
//     }

//     let text = resp
//         .text()
//         .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
//     let rtsp = text.trim().to_string();
//     if rtsp.is_empty() {
//         return Err(io::Error::new(io::ErrorKind::Other, "empty rtsp returned"));
//     }

//     // 新增：将camera_id和rtsp串写入rtsp.txt
//     if let Ok(mut file) = OpenOptions::new()
//         .create(true)
//         .append(true)
//         .open("rtsp.txt")
//     {
//         let _ = writeln!(file, "{},{}", camera_id, rtsp);
//     }
//     Ok(rtsp)
// }

pub fn get_live_url(camera_id: &str) -> Result<String, io::Error> {
    let url = format!("http://192.168.1.214:8686/media/getVideoStreaming");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let body = format!(r#"{{"cameraIndexCode":"{}","protocol":"rtsp"}}"#, camera_id);
    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    if !resp.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("HTTP status: {}", resp.status()),
        ));
    }
    let text = resp
        .text()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    println!("get_live_url resp: {}", text);
    // 解析JSON，提取data.url
    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Invalid JSON: {}", e)))?;
    println!("serde_json value: {:#?}", v);

    if v.get("code").and_then(|c| c.as_i64()) == Some(200) {
        if let Some(url) = v
            .get("data")
            .and_then(|d| d.get("url"))
            .and_then(|u| u.as_str())
        {
            // 将rtsp串写入rtsp.txt
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("rtsp.txt")
            {
                let _ = writeln!(file, "{}", url);
            }
            return Ok(url.to_string());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("Unexpected response: {}", text),
    ))
}
