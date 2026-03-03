//! 多身份隔离：按 profile 使用独立数据目录，实现 Cookie/LocalStorage 物理隔离。

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

const CURRENT_PROFILE_FILENAME: &str = "current_profile";
const PROFILES_SUBDIR: &str = "profiles";

/// 仅允许字母、数字、下划线，避免路径注入
fn sanitize_profile_name(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    let s = s.trim();
    if s.is_empty() {
        "default".to_string()
    } else {
        s.to_string()
    }
}

/// 读取当前 profile 名称，默认 "default"
fn get_current_profile_impl(app: &AppHandle) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = dir.join(CURRENT_PROFILE_FILENAME);
    let name = std::fs::read_to_string(&path).unwrap_or_else(|_| "default".to_string());
    let name = name.trim().to_string();
    Ok(if name.is_empty() {
        "default".to_string()
    } else {
        sanitize_profile_name(&name)
    })
}

#[tauri::command]
pub fn get_current_profile(app: AppHandle) -> Result<String, String> {
    get_current_profile_impl(&app)
}

/// 写入当前 profile；名称会做安全过滤
#[tauri::command]
pub fn set_current_profile(app: AppHandle, name: String) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let safe = sanitize_profile_name(&name);
    let path = dir.join(CURRENT_PROFILE_FILENAME);
    std::fs::write(path, safe).map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取当前 profile 对应的 webview 数据目录（profiles/<name>），并确保存在
pub fn profile_webview_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let name = get_current_profile_impl(app)?;
    let dir = base.join(PROFILES_SUBDIR).join(name);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}
