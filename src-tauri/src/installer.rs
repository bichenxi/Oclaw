//! fnm sidecar 安装 OpenClaw 的完整流程。
//! 步骤：
//!   1. fnm install 22（安装 Node.js 22）
//!   2. fnm exec --using=22 -- npm install -g openclaw
//!   3. fnm exec --using=22 -- openclaw onboard

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::ShellExt;

// ─── 状态 ──────────────────────────────────────────────────────────────────

pub struct InstallerState {
    running: Arc<Mutex<bool>>,
    cancel_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl Default for InstallerState {
    fn default() -> Self {
        Self {
            running: Arc::new(Mutex::new(false)),
            cancel_tx: Arc::new(Mutex::new(None)),
        }
    }
}

// ─── 事件 Payload ──────────────────────────────────────────────────────────

#[derive(Clone, serde::Serialize)]
struct StepPayload {
    step: String,
    status: String, // "running" | "done" | "error"
}

#[derive(Clone, serde::Serialize)]
struct LogPayload {
    line: String,
}

#[derive(Clone, serde::Serialize)]
struct ErrorPayload {
    step: String,
    message: String,
}

// ─── 辅助：emit helpers ────────────────────────────────────────────────────

fn emit_step(app: &AppHandle, step: &str, status: &str) {
    let _ = app.emit(
        "installer:step",
        StepPayload {
            step: step.to_string(),
            status: status.to_string(),
        },
    );
}

fn emit_log(app: &AppHandle, line: &str) {
    let _ = app.emit("installer:log", LogPayload { line: line.to_string() });
}

fn emit_error(app: &AppHandle, step: &str, message: &str) {
    let _ = app.emit(
        "installer:error",
        ErrorPayload {
            step: step.to_string(),
            message: message.to_string(),
        },
    );
}

// ─── 辅助：运行单步命令 ────────────────────────────────────────────────────

/// 在 `fnm_dir` 下运行 fnm sidecar，参数由调用方提供。
/// 实时把 stdout/stderr 通过 `installer:log` 事件推送到前端。
/// 成功返回 Ok(()), 取消或失败返回 Err(message)。
async fn run_step(
    app: &AppHandle,
    fnm_dir: &str,
    args: &[&str],
    cancel_rx: &mut tokio::sync::oneshot::Receiver<()>,
) -> Result<(), String> {
    use tauri_plugin_shell::process::CommandEvent;

    let shell = app.shell();
    let mut cmd = shell.sidecar("fnm").map_err(|e| e.to_string())?;
    cmd = cmd.args(["--fnm-dir", fnm_dir]);
    cmd = cmd.args(args);

    let (mut rx, child) = cmd.spawn().map_err(|e| e.to_string())?;

    loop {
        tokio::select! {
            // 取消信号
            _ = &mut *cancel_rx => {
                let _ = child.kill();
                return Err("已取消".to_string());
            }
            // 子进程输出
            maybe_event = rx.recv() => {
                match maybe_event {
                    Some(CommandEvent::Stdout(bytes)) => {
                        let line = String::from_utf8_lossy(&bytes);
                        emit_log(app, line.trim_end());
                    }
                    Some(CommandEvent::Stderr(bytes)) => {
                        let line = String::from_utf8_lossy(&bytes);
                        emit_log(app, line.trim_end());
                    }
                    Some(CommandEvent::Terminated(payload)) => {
                        let code = payload.code.unwrap_or(-1);
                        if code == 0 {
                            return Ok(());
                        } else {
                            return Err(format!("进程退出码 {}", code));
                        }
                    }
                    Some(_) => {}
                    None => {
                        return Ok(());
                    }
                }
            }
        }
    }
}

// ─── Tauri Command: start_install ─────────────────────────────────────────

#[tauri::command]
pub async fn start_install(app: AppHandle) -> Result<(), String> {
    let state = app
        .try_state::<InstallerState>()
        .ok_or("InstallerState not found")?;

    // 防重入
    {
        let mut running = state.running.lock().unwrap();
        if *running {
            return Err("安装已在进行中".to_string());
        }
        *running = true;
    }

    // 创建取消 channel
    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();
    {
        let mut guard = state.cancel_tx.lock().unwrap();
        *guard = Some(tx);
    }

    // fnm 数据目录
    let fnm_dir = app
        .path()
        .app_data_dir()
        .map(|p| p.join("fnm"))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            // 回退：使用 $HOME/.local/share/claw-browser/fnm
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{home}/.local/share/claw-browser/fnm")
        });

    // 克隆 app handle 供异步任务使用
    let app2 = app.clone();
    let running2 = state.running.clone();
    let cancel_tx2 = state.cancel_tx.clone();

    tauri::async_runtime::spawn(async move {
        let result = run_install_steps(&app2, &fnm_dir, &mut rx).await;

        // 清理状态
        {
            let mut r = running2.lock().unwrap();
            *r = false;
        }
        {
            let mut guard = cancel_tx2.lock().unwrap();
            *guard = None;
        }

        match result {
            Ok(()) => {
                let _ = app2.emit("installer:done", ());
            }
            Err(msg) => {
                // run_install_steps 内部已发送了 installer:error，这里不重复
                // 但若是取消信号返回的 Err，补一条 log
                if msg == "已取消" {
                    emit_log(&app2, "安装已取消。");
                }
            }
        }
    });

    Ok(())
}

/// 顺序执行全部安装步骤，任意一步失败则终止并返回 Err。
async fn run_install_steps(
    app: &AppHandle,
    fnm_dir: &str,
    cancel_rx: &mut tokio::sync::oneshot::Receiver<()>,
) -> Result<(), String> {
    // 步骤 1：安装 Node.js 22
    let step1 = "install-node";
    emit_step(app, step1, "running");
    emit_log(app, "正在安装 Node.js 22，首次下载约需 1~2 分钟...");
    match run_step(app, fnm_dir, &["install", "22"], cancel_rx).await {
        Ok(()) => emit_step(app, step1, "done"),
        Err(e) => {
            emit_step(app, step1, "error");
            emit_error(app, step1, &e);
            return Err(e);
        }
    }

    // 步骤 2：安装 openclaw（npm install -g）
    let step2 = "install-openclaw";
    emit_step(app, step2, "running");
    emit_log(app, "正在通过 npm 安装 openclaw，请稍候...");
    match run_step(
        app,
        fnm_dir,
        &["exec", "--using=22", "--", "npm", "install", "-g", "openclaw"],
        cancel_rx,
    )
    .await
    {
        Ok(()) => emit_step(app, step2, "done"),
        Err(e) => {
            emit_step(app, step2, "error");
            emit_error(app, step2, &e);
            return Err(e);
        }
    }

    // 步骤 3：openclaw onboard（初始化配置 / 启动 gateway）
    let step3 = "onboard";
    emit_step(app, step3, "running");
    emit_log(app, "正在初始化 OpenClaw 配置...");
    match run_step(
        app,
        fnm_dir,
        &["exec", "--using=22", "--", "openclaw", "onboard"],
        cancel_rx,
    )
    .await
    {
        Ok(()) => emit_step(app, step3, "done"),
        Err(e) => {
            emit_step(app, step3, "error");
            emit_error(app, step3, &e);
            return Err(e);
        }
    }

    Ok(())
}

// ─── Tauri Command: check_openclaw_installed ──────────────────────────────

/// 检测 OpenClaw 是否已安装并完成 onboard。
/// 判据：~/.openclaw/openclaw.json 存在。
#[tauri::command]
pub fn check_openclaw_installed(app: AppHandle) -> bool {
    if let Ok(home) = app.path().home_dir() {
        return home.join(".openclaw").join("openclaw.json").exists();
    }
    false
}

// ─── Tauri Command: cancel_install ────────────────────────────────────────

#[tauri::command]
pub async fn cancel_install(app: AppHandle) -> Result<(), String> {
    let state = app
        .try_state::<InstallerState>()
        .ok_or("InstallerState not found")?;
    let mut guard = state.cancel_tx.lock().unwrap();
    if let Some(tx) = guard.take() {
        let _ = tx.send(());
    }
    Ok(())
}
