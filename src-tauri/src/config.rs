// 前端 header 分两层：
//   - 身份行（Profile row）: py-1.5 + 按钮 ≈ 38px
//   - TabBar:               h-11 = 44px
// 合计约 82px；macOS 下子 webview 的 y 相对整窗（含标题栏约 44px），
// 故需再加约 44px 的标题栏偏移，取 128 留少量安全余量。
pub const TAB_BAR_HEIGHT: f64 = 128.0;
// 已移除左侧 AI 控制台，webview 占满全宽
pub const LEFT_PANEL_WIDTH: f64 = 0.0;

// Chromium-compatible User-Agent for child webviews.
// Mimics a real Chrome on macOS to avoid anti-bot detection by sites like Xiaohongshu.
pub const CHROME_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
