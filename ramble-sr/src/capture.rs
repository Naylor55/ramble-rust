use xcap::Monitor;

/// 屏幕捕获模块，封装 xcap 的显示器截屏功能
pub struct ScreenCapture {
    monitor: Monitor,
    width: u32,
    height: u32,
}

impl ScreenCapture {
    /// 创建屏幕捕获实例，monitor_index 指定显示器索引（0 为主显示器）
    pub fn new(monitor_index: usize) -> Result<Self, String> {
        let monitors = Monitor::all().map_err(|e| format!("获取显示器列表失败: {}", e))?;
        if monitor_index >= monitors.len() {
            return Err(format!(
                "显示器索引 {} 超出范围，共检测到 {} 个显示器",
                monitor_index,
                monitors.len()
            ));
        }
        let monitor = monitors[monitor_index].clone();
        let width = monitor.width().map_err(|e| format!("获取宽度失败: {}", e))?;
        let height = monitor.height().map_err(|e| format!("获取高度失败: {}", e))?;
        let name = monitor.name().unwrap_or_else(|_| "未知".to_string());

        println!("已选择显示器: {} ({}x{})", name, width, height);

        Ok(Self {
            monitor,
            width,
            height,
        })
    }

    /// 列出所有可用显示器
    pub fn list_monitors() -> Result<(), String> {
        let monitors = Monitor::all().map_err(|e| format!("获取显示器列表失败: {}", e))?;
        println!("检测到 {} 个显示器:", monitors.len());
        for (i, m) in monitors.iter().enumerate() {
            let name = m.name().unwrap_or_else(|_| "未知".to_string());
            let w = m.width().unwrap_or(0);
            let h = m.height().unwrap_or(0);
            let primary = m.is_primary().unwrap_or(false);
            println!(
                "  [{}] {} ({}x{}){}",
                i,
                name,
                w,
                h,
                if primary { " ★主显示器" } else { "" }
            );
        }
        Ok(())
    }

    /// 截取一帧，返回 RGBA 格式的像素数据
    pub fn capture_frame(&self) -> Result<image::RgbaImage, String> {
        self.monitor
            .capture_image()
            .map_err(|e| format!("截屏失败: {}", e))
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}
