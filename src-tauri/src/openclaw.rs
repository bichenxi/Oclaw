//! 通过 OpenClaw Gateway WebSocket API 连接，收发 chat、展示 response/tool_call 流。

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::mpsc;

const DEFAULT_GATEWAY_URL: &str = "ws://127.0.0.1:18789";

#[derive(Clone)]
pub enum OpenClawCommand {
    Connect { url: String, token: Option<String> },
    SendChat { text: String },
    Disconnect,
}

pub struct OpenClawState {
    tx: Mutex<Option<mpsc::UnboundedSender<OpenClawCommand>>>,
}

impl Default for OpenClawState {
    fn default() -> Self {
        Self {
            tx: Mutex::new(None),
        }
    }
}

#[derive(Deserialize)]
struct GatewayMessage {
    #[serde(rename = "type")]
    msg_type: String,
    payload: Option<serde_json::Value>,
}

fn emit_stream(app: &AppHandle, type_: &str, text: String) {
    let _ = app.emit(
        "stream-item",
        serde_json::json!({ "type": type_, "text": text }),
    );
}

fn emit_status(app: &AppHandle, status: &str) {
    let _ = app.emit("openclaw-status", status);
}

fn build_ws_url(url: &str, _token: Option<&str>) -> String {
    // Gateway 认证通过握手时的 auth.token 传递，不在 URL 里加 token
    url.trim().trim_end_matches('/').to_string()
}

fn gateway_platform() -> &'static str {
    match std::env::consts::OS {
        "macos" | "darwin" => "macos",
        "windows" => "windows",
        "linux" => "linux",
        _ => "linux",
    }
}

/// OpenClaw Gateway 要求：client.id / client.mode 必须为 GATEWAY_CLIENT_IDS / GATEWAY_CLIENT_MODES 枚举值（无 "operator"，用 "cli"）
fn build_connect_req(token: Option<&str>) -> serde_json::Value {
    let mut params = serde_json::json!({
        "minProtocol": 3,
        "maxProtocol": 3,
        "client": {
            "id": "cli",
            "version": "0.1.0",
            "platform": gateway_platform(),
            "mode": "cli"
        },
        "role": "operator",
        "scopes": ["operator.read", "operator.write"],
        "locale": "zh-CN"
    });
    if let Some(t) = token {
        params["auth"] = serde_json::json!({ "token": t });
    }
    serde_json::json!({
        "type": "req",
        "id": format!("connect-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "method": "connect",
        "params": params
    })
}

fn reader_loop(
    app: AppHandle,
    mut read: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
) {
    tauri::async_runtime::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(tokio_tungstenite::tungstenite::Message::Text(s)) => {
                    if let Ok(parsed) = serde_json::from_str::<GatewayMessage>(&s) {
                        match parsed.msg_type.as_str() {
                            "response" => {
                                if let Some(p) = parsed.payload {
                                    if let Some(t) = p.get("text").and_then(|v| v.as_str()) {
                                        emit_stream(&app, "thought", t.to_string());
                                    }
                                }
                            }
                            "res" => {
                                // 请求的回复帧：payload 中可能有 text（agent 回复）
                                if let Some(p) = parsed.payload {
                                    if let Some(t) = p.get("text").and_then(|v| v.as_str()) {
                                        if !t.is_empty() {
                                            emit_stream(&app, "thought", t.to_string());
                                        }
                                    }
                                }
                            }
                            "event" => {
                                // 事件帧：event 字段在 payload 或需从外层解析，常见为 agent 流式输出
                                if let Some(p) = &parsed.payload {
                                    if let Some(t) = p.get("text").and_then(|v| v.as_str()) {
                                        if !t.is_empty() {
                                            emit_stream(&app, "thought", t.to_string());
                                        }
                                    }
                                }
                            }
                            "tool_call" => {
                                if let Some(p) = parsed.payload {
                                    let tool =
                                        p.get("tool").and_then(|v| v.as_str()).unwrap_or("?");
                                    let args = p
                                        .get("args")
                                        .map(|v| v.to_string())
                                        .unwrap_or_else(|| "{}".to_string());
                                    emit_stream(&app, "tool", format!("{} {}", tool, args));
                                }
                            }
                            "tool_result" => {
                                if let Some(p) = parsed.payload {
                                    let out = p
                                        .get("output")
                                        .map(|v| v.to_string())
                                        .unwrap_or_else(|| p.to_string());
                                    emit_stream(&app, "tool", format!("result: {}", out));
                                }
                            }
                            "error" => {
                                if let Some(p) = parsed.payload {
                                    let msg = p
                                        .get("message")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("error");
                                    emit_stream(&app, "thought", format!("OpenClaw 错误: {}", msg));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                    emit_status(&app, "disconnected");
                    break;
                }
                Err(_) | Ok(_) => {
                    emit_status(&app, "disconnected");
                    break;
                }
            }
        }
    });
}

pub fn run_gateway_task(app: AppHandle, mut rx: mpsc::UnboundedReceiver<OpenClawCommand>) {
    tauri::async_runtime::spawn(async move {
        type WsWrite = futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            tokio_tungstenite::tungstenite::Message,
        >;
        let mut ws_write: Option<WsWrite> = None;

        while let Some(cmd) = rx.recv().await {
            match cmd {
                OpenClawCommand::Disconnect => {
                    ws_write = None;
                    emit_status(&app, "disconnected");
                }
                OpenClawCommand::Connect { url, token } => {
                    let full = build_ws_url(&url, token.as_deref());
                    match tokio_tungstenite::connect_async(&full).await {
                        Ok((stream, _)) => {
                            let (mut write, mut read) = stream.split();
                            // Gateway 握手：先读一帧（connect.challenge 或首帧），再发 connect 请求
                            let _first = read.next().await;
                            let connect_req = build_connect_req(token.as_deref());
                            if write
                                .send(tokio_tungstenite::tungstenite::Message::Text(
                                    connect_req.to_string(),
                                ))
                                .await
                                .is_err()
                            {
                                emit_status(&app, "error");
                                emit_stream(&app, "thought", "握手发送 connect 失败".to_string());
                                continue;
                            }
                            // 再读一帧（hello-ok 或 res），若为错误则提示
                            match read.next().await {
                                Some(Ok(tokio_tungstenite::tungstenite::Message::Text(s))) => {
                                    if s.contains("error") || s.contains("AUTH") {
                                        emit_status(&app, "error");
                                        emit_stream(&app, "thought", format!("握手失败: {}", s));
                                        continue;
                                    }
                                }
                                Some(Err(e)) => {
                                    emit_status(&app, "error");
                                    emit_stream(&app, "thought", format!("握手读响应失败: {}", e));
                                    continue;
                                }
                                _ => {}
                            }
                            reader_loop(app.clone(), read);
                            ws_write = Some(write);
                            emit_status(&app, "connected");
                        }
                        Err(e) => {
                            emit_status(&app, "error");
                            emit_stream(&app, "thought", format!("连接失败: {}", e));
                        }
                    }
                }
                OpenClawCommand::SendChat { text } => {
                    if let Some(ref mut w) = ws_write {
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                        let id = format!("chat-{}", ts);
                        // Gateway 协议：请求用 type "req" + method "chat.send"，回复为 type "res"
                        let msg = serde_json::json!({
                            "type": "req",
                            "id": id,
                            "method": "chat.send",
                            "params": {
                                "text": text,
                                "context": {},
                                "options": {}
                            }
                        });
                        if w.send(tokio_tungstenite::tungstenite::Message::Text(
                            msg.to_string(),
                        ))
                        .await
                        .is_err()
                        {
                            ws_write = None;
                            emit_status(&app, "disconnected");
                        }
                    } else {
                        emit_stream(&app, "thought", "未连接 OpenClaw，请先连接。".to_string());
                    }
                }
            }
        }
    });
}

fn ensure_tx(app: &AppHandle, state: &OpenClawState) -> mpsc::UnboundedSender<OpenClawCommand> {
    let mut guard = state.tx.lock().unwrap();
    if guard.is_none() {
        let (tx, rx) = mpsc::unbounded_channel();
        run_gateway_task(app.clone(), rx);
        let tx_clone = tx.clone();
        *guard = Some(tx);
        tx_clone
    } else {
        guard.as_ref().unwrap().clone()
    }
}

#[tauri::command]
pub fn openclaw_connect(
    app: AppHandle,
    state: State<'_, OpenClawState>,
    url: Option<String>,
    token: Option<String>,
) -> Result<(), String> {
    let url = url.unwrap_or_else(|| DEFAULT_GATEWAY_URL.to_string());
    let tx = ensure_tx(&app, &state);
    tx.send(OpenClawCommand::Connect { url, token })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn openclaw_send_chat(
    app: AppHandle,
    state: State<'_, OpenClawState>,
    text: String,
) -> Result<(), String> {
    let tx = ensure_tx(&app, &state);
    tx.send(OpenClawCommand::SendChat { text })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn openclaw_disconnect(state: State<'_, OpenClawState>) -> Result<(), String> {
    let guard = state.tx.lock().unwrap();
    if let Some(ref tx) = *guard {
        tx.send(OpenClawCommand::Disconnect)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
