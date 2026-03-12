use tauri::Emitter;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Oclaw ready.", name)
}

#[tauri::command]
pub async fn on_webview_click(label: String, x: i32, y: i32, target: String) {
    println!("Webview '{}' clicked at ({}, {}) on {}", label, x, y, target);
}

/// 供 OpenClaw Sidecar 或前端调试使用：向左侧控制台推送一条思考链/工具流条目
#[tauri::command]
pub fn emit_stream_item(app: tauri::AppHandle, type_: String, text: String) {
    let payload = serde_json::json!({ "type": type_, "text": text });
    let _ = app.emit("stream-item", payload);
}

/// 模拟流式输出，用于验证「指令解析与可视化流」UI 与事件链路
#[tauri::command]
pub async fn simulate_stream(app: tauri::AppHandle) {
    use tokio::time::{sleep, Duration};
    let items: Vec<(&str, &str)> = vec![
        ("thought", "正在分析当前页面结构…"),
        ("thought", "发现搜索框与导航链接，优先执行导航。"),
        ("tool", "navigate https://example.com"),
        ("tool", "click selector: a[href=\"/demo\"]"),
        ("thought", "已到达目标页，等待 DOM 稳定后提取快照。"),
    ];
    for (type_, text) in items {
        let _ = app.emit("stream-item", serde_json::json!({ "type": type_, "text": text }));
        sleep(Duration::from_millis(450)).await;
    }
}
