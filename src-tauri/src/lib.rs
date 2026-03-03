mod app;
mod config;
mod profile;
mod sidecar;
mod webview;

use tauri::generate_handler;
use app::{emit_stream_item, simulate_stream};
use profile::{get_current_profile, set_current_profile};
use webview::commands::{
    close_webview, create_tab_webview, eval_in_webview, get_dom_snapshot, hide_webview,
    resize_all_webviews, show_webview,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(generate_handler![
            app::greet,
            app::on_webview_click,
            emit_stream_item,
            simulate_stream,
            get_current_profile,
            set_current_profile,
            sidecar::test_sidecar,
            create_tab_webview,
            show_webview,
            hide_webview,
            close_webview,
            resize_all_webviews,
            eval_in_webview,
            get_dom_snapshot,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
