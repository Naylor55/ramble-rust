# screen-recorder 🎥

Windows 录屏工具，使用 Rust 编写。通过 `xcap` 捕获屏幕帧，管道输送给 FFmpeg 编码为视频文件。

## 功能

- ✅ 全屏录制（支持多显示器选择）
- ✅ H.264 编码输出 MP4
- ✅ 可配置帧率
- ✅ Ctrl+C 优雅停止
- ✅ 实时录制状态显示
- ✅ 自动检测 FFmpeg 环境

## 前置依赖

1. **Rust** — [安装 Rust](https://www.rust-lang.org/tools/install)
2. **FFmpeg** — 需要安装并添加到系统 PATH
   - 下载: https://ffmpeg.org/download.html
   - Windows 推荐下载预编译的 shared build，解压后将 `bin/` 目录加入 PATH
3. **Windows 10 1903+** — `xcap` 依赖 Windows Graphics Capture API

## 编译

```bash
cargo build --release
```

## 使用

```bash
# 使用默认参数录制主显示器（30fps → output.mp4）
screen-recorder.exe

# 指定输出文件和帧率
screen-recorder.exe -o recording.mp4 -f 60

# 录制第二个显示器
screen-recorder.exe --monitor 1

# 列出所有显示器
screen-recorder.exe --list
```

### 命令行参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `-o, --output` | 输出文件路径 | `output.mp4` |
| `-f, --fps` | 目标帧率 | `30` |
| `-m, --monitor` | 显示器索引 | `0`（主显示器）|
| `-l, --list` | 列出所有显示器 | — |

### 停止录制

录制过程中按 **Ctrl+C** 即可停止，程序会自动完成编码并保存文件。

## 项目结构

```
src/
├── main.rs       # 入口，CLI 参数解析，信号处理
├── capture.rs    # 屏幕捕获（xcap 封装）
├── encoder.rs    # 视频编码（FFmpeg 子进程）
└── recorder.rs   # 录制控制（帧循环、状态管理）
```

## 工作原理

```
屏幕帧 (xcap) → RGBA 原始数据 → FFmpeg stdin → H.264 编码 → MP4 文件
```

1. `xcap` 调用 Windows Graphics Capture API 截取屏幕
2. 原始 RGBA 像素数据通过管道写入 FFmpeg 子进程
3. FFmpeg 使用 `libx264 ultrafast` 预设实时编码
4. Ctrl+C 信号关闭管道，FFmpeg 完成编码后输出文件

## 后续可扩展

- [ ] 录制系统音频（WASAPI loopback）
- [ ] 区域录制（指定坐标范围）
- [ ] 鼠标光标捕获
- [ ] GUI 界面（系统托盘控制）
- [ ] 纯 Rust 编码（去掉 FFmpeg 依赖）
- [ ] GPU 硬件加速编码（NVENC/QSV）
