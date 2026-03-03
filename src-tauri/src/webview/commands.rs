use base64::Engine;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl};

use crate::profile;
use crate::webview::rect;

#[tauri::command]
pub async fn create_tab_webview(app: AppHandle, label: String, url: String) -> Result<(), String> {
    let window = app.get_window("main").ok_or("main window not found")?;

    let parsed_url: url::Url = url.parse().map_err(|e: url::ParseError| e.to_string())?;
    let (x, y, w, h) = rect::calc_webview_rect(&window)?;

    let bridge_js = include_str!("../bridge.js");
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

    let data_dir: PathBuf = profile::profile_webview_data_dir(&app)?;
    let builder = tauri::webview::WebviewBuilder::new(&label, WebviewUrl::External(parsed_url))
        .initialization_script(&init_script)
        .on_navigation(nav_handler)
        .data_directory(data_dir);

    window
        .add_child(builder, LogicalPosition::new(x, y), LogicalSize::new(w, h))
        .map_err(|e: tauri::Error| e.to_string())?;

    if let Some(wv) = app.get_webview(&label) {
        let _ = wv.hide();
    }
    Ok(())
}

#[tauri::command]
pub async fn show_webview(app: AppHandle, label: String) -> Result<(), String> {
    let wv = app
        .get_webview(&label)
        .ok_or(format!("webview '{}' not found", label))?;
    wv.show().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn hide_webview(app: AppHandle, label: String) -> Result<(), String> {
    let wv = app
        .get_webview(&label)
        .ok_or(format!("webview '{}' not found", label))?;
    wv.hide().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn close_webview(app: AppHandle, label: String) -> Result<(), String> {
    let wv = app
        .get_webview(&label)
        .ok_or(format!("webview '{}' not found", label))?;
    wv.close().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn resize_all_webviews(app: AppHandle, labels: Vec<String>) -> Result<(), String> {
    let window = app.get_window("main").ok_or("main window not found")?;
    let (x, y, w, h) = rect::calc_webview_rect(&window)?;

    for label in labels {
        if let Some(wv) = app.get_webview(&label) {
            let _ = wv.set_position(LogicalPosition::new(x, y));
            let _ = wv.set_size(LogicalSize::new(w, h));
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn eval_in_webview(app: AppHandle, label: String, script: String) -> Result<(), String> {
    let wv = app
        .get_webview(&label)
        .ok_or(format!("webview '{}' not found", label))?;
    wv.eval(&script).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_dom_snapshot(app: AppHandle, label: String) -> Result<(), String> {
    let wv = app
        .get_webview(&label)
        .ok_or(format!("webview '{}' not found", label))?;

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
