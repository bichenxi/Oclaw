use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

#[tauri::command]
pub async fn test_sidecar(app: AppHandle) -> Result<String, String> {
    let shell = app.shell();
    let sidecar = shell.sidecar("openclaw").map_err(|e| e.to_string())?;

    let (mut rx, mut child) = sidecar.spawn().map_err(|e| e.to_string())?;

    child
        .write(b"hello from tauri\n")
        .map_err(|e| e.to_string())?;

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
