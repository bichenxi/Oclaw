use base64::Engine;
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl};
use tauri_plugin_shell::ShellExt;

// 44px 为前端 TabBar 高度；macOS 下子 webview 的 y 可能相对整窗（含标题栏），多留余量避免遮挡
const TAB_BAR_HEIGHT: f64 = 88.0;
// 阶段 1 分屏：左侧 AI 控制台宽度，与前端 .layout-sidebar 一致
const LEFT_PANEL_WIDTH: f64 = 320.0;

fn calc_webview_rect(window: &tauri::Window) -> Result<(f64, f64, f64, f64), String> {
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let inner = window.inner_size().map_err(|e| e.to_string())?;
    let total_w = inner.width as f64 / scale;
    let h = inner.height as f64 / scale;
    let w = (total_w - LEFT_PANEL_WIDTH).max(0.0);
    Ok((LEFT_PANEL_WIDTH, TAB_BAR_HEIGHT, w, h - TAB_BAR_HEIGHT))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Claw Browser ready.", name)
}

#[tauri::command]
async fn test_sidecar(app: AppHandle) -> Result<String, String> {
    let shell = app.shell();
    let sidecar = shell.sidecar("openclaw").map_err(|e| e.to_string())?;

    let (mut rx, mut child) = sidecar.spawn().map_err(|e| e.to_string())?;

    // 发送消息到 sidecar stdin
    child
        .write(b"hello from tauri\n")
        .map_err(|e| e.to_string())?;

    // 读取 sidecar stdout 响应
    while let Some(event) = rx.recv().await {
        if let tauri_plugin_shell::process::CommandEvent::Stdout(line) = event {
            let res = String::from_utf8_lossy(&line).to_string();
            if res.contains("ACK:") {
                let _ = child.kill();
                return Ok(res);
            }
        }
    }

    Err("sidecar failed to respond".to_string())
}

#[tauri::command]
async fn on_webview_click(label: String, x: i32, y: i32, target: String) {
    println!("Webview '{}' clicked at ({}, {}) on {}", label, x, y, target);
}

#[tauri::command]
async fn create_tab_webview(app: AppHandle, label: String, url: String) -> Result<(), String> {
    let window = app.get_window("main").ok_or("main window not found")?;

    let parsed_url: url::Url = url.parse().map_err(|e: url::ParseError| e.to_string())?;
    let (x, y, w, h) = calc_webview_rect(&window)?;

    // 注入 bridge.js 并注入当前 webview 的 label；点击通过 claw:// 触发，由 on_navigation 拦截
    let bridge_js = include_str!("bridge.js");
    let init_script = format!(
        "{};\nwindow.__clawBridgeLabel={:?};",
        bridge_js.trim_end().trim_end_matches(';'),
        label
    );

    let app_emit = app.clone();
    let nav_handler = move |nav_url: &url::Url| {
        if nav_url.scheme() != "claw" {
            return true;
        }
        if nav_url.host_str() == Some("webview-click") {
            let mut label_val = String::new();
            let mut x_val = 0i32;
            let mut y_val = 0i32;
            let mut tag_val = String::new();
            for (k, v) in nav_url.query_pairs() {
                match k.as_ref() {
                    "label" => label_val = v.to_string(),
                    "x" => x_val = v.parse().unwrap_or(0),
                    "y" => y_val = v.parse().unwrap_or(0),
                    "tag" => tag_val = v.to_string(),
                    _ => {}
                }
            }
            println!(
                "Webview '{}' clicked at ({}, {}) on {}",
                label_val, x_val, y_val, tag_val
            );
            return false;
        }
        if nav_url.host_str() == Some("dom-snapshot") {
            if let Some(frag) = nav_url.fragment() {
                let b64 = frag.replace('-', "+").replace('_', "/");
                if let Ok(decoded) =
                    base64::engine::general_purpose::STANDARD.decode(b64.as_bytes())
                {
                    if let Ok(s) = String::from_utf8(decoded) {
                        let _ = app_emit.emit("dom-snapshot", &s);
                    }
                }
            }
            return false;
        }
        true
    };

    window
        .add_child(
            tauri::webview::WebviewBuilder::new(&label, WebviewUrl::External(parsed_url))
                .initialization_script(&init_script)
                .on_navigation(nav_handler),
            LogicalPosition::new(x, y),
            LogicalSize::new(w, h),
        )
        .map_err(|e: tauri::Error| e.to_string())?;

    // 先隐藏，等前端展示完加载动画后再 show
    if let Some(wv) = app.get_webview(&label) {
        let _ = wv.hide();
    }
    Ok(())
}

#[tauri::command]
async fn show_webview(app: AppHandle, label: String) -> Result<(), String> {
    let wv = app
        .get_webview(&label)
        .ok_or(format!("webview '{}' not found", label))?;
    wv.show().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn hide_webview(app: AppHandle, label: String) -> Result<(), String> {
    let wv = app
        .get_webview(&label)
        .ok_or(format!("webview '{}' not found", label))?;
    wv.hide().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn close_webview(app: AppHandle, label: String) -> Result<(), String> {
    let wv = app
        .get_webview(&label)
        .ok_or(format!("webview '{}' not found", label))?;
    wv.close().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn resize_all_webviews(app: AppHandle, labels: Vec<String>) -> Result<(), String> {
    let window = app.get_window("main").ok_or("main window not found")?;

    let (x, y, w, h) = calc_webview_rect(&window)?;

    for label in labels {
        if let Some(wv) = app.get_webview(&label) {
            let _ = wv.set_position(LogicalPosition::new(x, y));
            let _ = wv.set_size(LogicalSize::new(w, h));
        }
    }
    Ok(())
}

#[tauri::command]
async fn eval_in_webview(app: AppHandle, label: String, script: String) -> Result<(), String> {
    let wv = app
        .get_webview(&label)
        .ok_or(format!("webview '{}' not found", label))?;
    wv.eval(&script).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_dom_snapshot(app: AppHandle, label: String) -> Result<(), String> {
    let wv = app
        .get_webview(&label)
        .ok_or(format!("webview '{}' not found", label))?;

    // 用主框架 location 触发 claw://，由 on_navigation 拦截并 cancel，避免 iframe 的 frame-src 与 HTTP 的 Mixed Content
    // 数据放 fragment，长度限制约 1.5e6 字符以防 URL 超长
    let script = r#"
(function(){
  var data = window.__clawBridge && window.__clawBridge.getSimplifiedDOM ? window.__clawBridge.getSimplifiedDOM() : [];
  var json = JSON.stringify(data);
  if (json.length > 1500000) json = json.slice(0, 1500000) + ']';
  var b64 = btoa(unescape(encodeURIComponent(json))).replace(/\+/g, '-').replace(/\//g, '_');
  window.location.assign('claw://dom-snapshot#' + b64);
})();
"#;
    wv.eval(script).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            test_sidecar,
            on_webview_click,
            eval_in_webview,
            get_dom_snapshot,
            create_tab_webview,
            show_webview,
            hide_webview,
            close_webview,
            resize_all_webviews,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
