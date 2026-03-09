//! 在 Tauri 应用内启动/停止 OpenClaw 进程。
//! OpenClaw 是 Node.js 项目，可通过「系统 node + 入口脚本」或「pkg 打包后的 Sidecar 二进制」启动。

use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;

/// 持有 OpenClaw 子进程；应用退出时需主动 kill，见 `stop_openclaw`。
pub struct OpenClawProcess(pub Mutex<Option<CommandChild>>);

impl Default for OpenClawProcess {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

/// 环境变量：OpenClaw 入口脚本路径（如 /path/to/openclaw/dist/index.js）。
/// 若设置则用 `node $OPENCLAW_ENTRY` 启动；未设置则尝试 Sidecar 二进制。
const ENV_OPENCLAW_ENTRY: &str = "OPENCLAW_ENTRY";

/// 启动 OpenClaw。若已运行则返回错误。
#[tauri::command]
pub async fn start_openclaw(app: AppHandle) -> Result<(), String> {
    let state = app
        .try_state::<OpenClawProcess>()
        .ok_or("OpenClawProcess state not found")?;
    {
        let guard = state.0.lock().expect("openclaw process lock");
        if guard.is_some() {
            return Err("OpenClaw 已在运行".to_string());
        }
    }

    let shell = app.shell();
    let child = if let Ok(entry) = std::env::var(ENV_OPENCLAW_ENTRY) {
        // 使用系统 node 运行入口脚本
        let cmd = shell.command("node").args([entry]);
        let (mut rx, child) = cmd.spawn().map_err(|e| e.to_string())?;
        tauri::async_runtime::spawn(async move {
            while rx.recv().await.is_some() {}
        });
        child
    } else {
        // 回退到 Sidecar 二进制（需已用 pkg 等打好 openclaw 并放入 bin/）
        let sidecar = shell.sidecar("openclaw").map_err(|e| e.to_string())?;
        let (mut rx, child) = sidecar.spawn().map_err(|e| e.to_string())?;
        tauri::async_runtime::spawn(async move {
            while rx.recv().await.is_some() {}
        });
        child
    };

    let mut guard = state.0.lock().expect("openclaw process lock");
    *guard = Some(child);
    Ok(())
}

/// 停止 OpenClaw 子进程。
#[tauri::command]
pub async fn stop_openclaw(app: AppHandle) -> Result<(), String> {
    let state = app
        .try_state::<OpenClawProcess>()
        .ok_or("OpenClawProcess state not found")?;
    let mut guard = state.0.lock().expect("openclaw process lock");
    if let Some(child) = guard.take() {
        let _ = child.kill();
    }
    Ok(())
}

/// 是否正在运行 OpenClaw。
#[tauri::command]
pub fn is_openclaw_running(app: AppHandle) -> Result<bool, String> {
    let state = app
        .try_state::<OpenClawProcess>()
        .ok_or("OpenClawProcess state not found")?;
    let guard = state.0.lock().expect("openclaw process lock");
    Ok(guard.is_some())
}
