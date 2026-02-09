# RTMP 流媒体服务器

一个用 Rust 编写的简单 RTMP/HTTP-FLV 流媒体辅助服务（开发中）。

## 功能概述

- RTMP 协议支持（进行中）
- 流管理
- 多客户端支持
- 可配置绑定地址与端口
- 结构化日志输出

## 要求

- Rust 1.70+（使用 2021 版）
- Cargo

## 构建

```bash
cd rtmp-streaming-server
cargo build --release
```

## 运行

```bash
# 使用默认设置（RTMP 占位 TCP:1935，HTTP-FLV:8080）
cargo run --release

# 自定义地址与日志级别（例如仍绑定 1935）
cargo run --release -- --address 0.0.0.0:1935 --log-level info

# 获取帮助
cargo run --release -- --help
```

## 使用 FFmpeg 测试（示例）

> 说明：本项目提供两个通路：一是占位的 RTMP TCP 监听（1935），二是可用的 HTTP-FLV 推/拉（8080）。当前已实现 HTTP-FLV 推流/拉流并对 H.264 做了头部/序列头转发支持，生产环境请使用成熟项目（如 nginx-rtmp / SRS）。

### 推流（将文件推到 HTTP-FLV 服务并转码为 H.264）

```bash
# 转码视频为 H.264，保留音频并推送到 HTTP-FLV
ffmpeg -re -i input.mp4 -c:v libx264 -preset veryfast -tune zerolatency -c:a aac -b:a 128k -f flv "http://localhost:8080/live/stream1"
```

### 禁用音频（推送仅视频）

```bash
# 方案一：在推流时直接丢弃音频并转码视频
ffmpeg -re -i input.mp4 -c:v libx264 -preset veryfast -tune zerolatency -an -f flv "http://localhost:8080/live/stream1"

# 方案二：只复制视频流并丢弃音频（无需重新编码视频）
ffmpeg -re -i input.mp4 -map 0:v -c:v copy -an -f flv "http://localhost:8080/live/stream1"
```

### 播放（拉流）

```bash
# 使用 ffplay 拉取 HTTP-FLV 流
ffplay "http://localhost:8080/live/stream1"

# 占位的 RTMP TCP（注意：目前未实现完整 RTMP 握手/消息处理，仅用于验证 TCP 可连接）
ffplay "rtmp://localhost:1935/live/stream1"
```

可以成功拉取到视频流并播放，得多等一会。



## 项目结构

```
src/
├── main.rs          # 程序主入口（包含 HTTP-FLV 与 RTMP 占位）
├── lib.rs           # 库导出（占位）
├── error.rs         # 错误类型与处理（占位）
├── server.rs        # RTMP 服务实现（WIP）
├── session.rs       # RTMP 会话处理（WIP）
├── stream.rs        # 流管理逻辑
└── protocol.rs      # RTMP 协议定义（WIP）
```

## 依赖（主要）

- rtmp — RTMP 协议实现（计划）
- flv — FLV 格式支持（计划/部分使用）
- amf — AMF 编解码
- tokio — 异步运行时
- clap — 命令行参数解析
- tracing — 结构化日志
- anyhow — 错误处理
- uuid — UUID 生成

## 开发进度

⚠️ 开发中（Work in Progress）

已实现/部分实现：

- [x] 基本工程结构
- [x] 错误处理框架
- [x] 服务框架（HTTP-FLV 可用于推/拉）
- [x] 流管理与简单转发（丢弃音频 tag）
- [x] 会话与协议代码框架（未完成）

待完成 / 计划中：

- [ ] 完整的 RTMP 握手实现
- [ ] RTMP 分块流处理（chunk）
- [ ] RTMP 命令处理（connect/publish/play）
- [ ] 媒体数据完整处理与转发策略
- [ ] 完善的客户端连接管理与测试

## 许可证

MIT