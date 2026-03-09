//! Xiaohongshu (Little Red Book) site-specific HTTP API endpoints.
//! Mounted under `/xhs` by the main API server.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use tauri::AppHandle;

use crate::api::{
    eval_with_result, get_active_label, internal_err, no_active_tab, to_js_str, ErrorResponse,
    EvalResponse,
};

// ── Request types ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchBody {
    pub keyword: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenNoteBody {
    pub index: u32,
}

#[derive(Debug, Deserialize)]
pub struct ResultsQuery {
    #[serde(default = "default_limit")]
    pub limit: Option<u32>,
}
fn default_limit() -> Option<u32> {
    None
}

// ── Helper ───────────────────────────────────────────────────────────────────

/// Evaluate a __clawXhs method call and parse the result into a JSON value.
async fn xhs_eval(
    app: AppHandle,
    label: String,
    script: &str,
) -> Result<serde_json::Value, (StatusCode, Json<ErrorResponse>)> {
    let result_str = eval_with_result(app, label, script.to_string())
        .await
        .map_err(internal_err)?;
    let resp: EvalResponse =
        serde_json::from_str(&result_str).map_err(|e| internal_err(e.to_string()))?;
    if !resp.ok {
        return Err(internal_err(
            resp.error.unwrap_or_else(|| "xhs eval failed".to_string()),
        ));
    }
    Ok(resp.value.unwrap_or(serde_json::Value::Null))
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /xhs/check-login
async fn handle_check_login(
    State(app): State<AppHandle>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let label = get_active_label(&app).ok_or_else(no_active_tab)?;
    let val = xhs_eval(app, label, "window.__clawXhs.isLoggedIn()").await?;
    Ok(Json(serde_json::json!({ "loggedIn": val })))
}

/// POST /xhs/search  { "keyword": "小米SU7" }
/// Sets the search input value and clicks search. Caller should /wait + /xhs/results after.
async fn handle_search(
    State(app): State<AppHandle>,
    Json(body): Json<SearchBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let label = get_active_label(&app).ok_or_else(no_active_tab)?;
    let set_script = format!(
        "window.__clawXhs.setSearchInput({})",
        to_js_str(&body.keyword)
    );
    let set_ok = xhs_eval(app.clone(), label.clone(), &set_script).await?;
    if set_ok == serde_json::Value::Bool(false) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Search input not found — is this a Xiaohongshu page?".to_string(),
            }),
        ));
    }
    // Small delay then click search
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let click_ok = xhs_eval(app, label, "window.__clawXhs.clickSearch()").await?;
    Ok(Json(
        serde_json::json!({ "inputSet": set_ok, "searchClicked": click_ok }),
    ))
}

/// GET /xhs/results?limit=20
async fn handle_results(
    State(app): State<AppHandle>,
    Query(params): Query<ResultsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let label = get_active_label(&app).ok_or_else(no_active_tab)?;
    let limit = params.limit.unwrap_or(20);
    let script = format!("window.__clawXhs.getResults({})", limit);
    let items = xhs_eval(app, label, &script).await?;
    Ok(Json(serde_json::json!({ "items": items })))
}

/// POST /xhs/open-note  { "index": 0 }
/// Clicks the note at the given feed index to open the overlay.
/// Caller should wait ~1.5 s then call /xhs/note-content.
async fn handle_open_note(
    State(app): State<AppHandle>,
    Json(body): Json<OpenNoteBody>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let label = get_active_label(&app).ok_or_else(no_active_tab)?;
    let script = format!("window.__clawXhs.openNote({})", body.index);
    let ok = xhs_eval(app, label, &script).await?;
    if ok == serde_json::Value::Bool(false) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Note at index {} not found", body.index),
            }),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// POST /xhs/close-note
async fn handle_close_note(
    State(app): State<AppHandle>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let label = get_active_label(&app).ok_or_else(no_active_tab)?;
    xhs_eval(app, label, "window.__clawXhs.closeNote()").await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /xhs/note-content
/// Reads the currently open note's title, content, author, stats, tags, and top comments.
async fn handle_note_content(
    State(app): State<AppHandle>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let label = get_active_label(&app).ok_or_else(no_active_tab)?;
    let note = xhs_eval(app, label, "window.__clawXhs.getNoteContent()").await?;
    Ok(Json(note))
}

/// GET /xhs/note-images
/// Returns image URLs from the current note's swiper/carousel.
async fn handle_note_images(
    State(app): State<AppHandle>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let label = get_active_label(&app).ok_or_else(no_active_tab)?;
    let images = xhs_eval(app, label, "window.__clawXhs.getNoteImages()").await?;
    Ok(Json(serde_json::json!({ "images": images })))
}

/// POST /xhs/scroll-feed
/// Scrolls down 800px to trigger lazy loading of more results.
async fn handle_scroll_feed(
    State(app): State<AppHandle>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let label = get_active_label(&app).ok_or_else(no_active_tab)?;
    xhs_eval(app, label, "window.__clawXhs.scrollFeed()").await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /xhs/is-note-open
async fn handle_is_note_open(
    State(app): State<AppHandle>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let label = get_active_label(&app).ok_or_else(no_active_tab)?;
    let open = xhs_eval(app, label, "window.__clawXhs.isNoteOpen()").await?;
    Ok(Json(serde_json::json!({ "open": open })))
}

// ── Router ───────────────────────────────────────────────────────────────────

pub fn xhs_router() -> Router<AppHandle> {
    Router::new()
        .route("/check-login", get(handle_check_login))
        .route("/search", post(handle_search))
        .route("/results", get(handle_results))
        .route("/open-note", post(handle_open_note))
        .route("/close-note", post(handle_close_note))
        .route("/note-content", get(handle_note_content))
        .route("/note-images", get(handle_note_images))
        .route("/scroll-feed", post(handle_scroll_feed))
        .route("/is-note-open", get(handle_is_note_open))
}
