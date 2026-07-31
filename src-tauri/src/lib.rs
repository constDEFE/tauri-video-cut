mod commands;
mod config;
mod core;
mod error;
mod logger;
mod session;
mod types;
mod utils;

use core::ProcessManager;
use core::waveform::{WaveformJobRegistry, cancel_waveform, stream_waveform};
use session::load_session;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use utils::cleanup::{cleanup_old_waveforms, cleanup_orphaned_temp_segments};
use utils::dll::add_lib_to_dll_search_path;

#[cfg(debug_assertions)]
fn prevent_default() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    use tauri_plugin_prevent_default::Flags;

    tauri_plugin_prevent_default::Builder::new()
        .with_flags(Flags::all().difference(Flags::DEV_TOOLS | Flags::RELOAD))
        .build()
}

#[cfg(not(debug_assertions))]
fn prevent_default() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_prevent_default::init()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logger::init();

    log_info!("VideoCut backend initializing");

    add_lib_to_dll_search_path();
    cleanup_orphaned_temp_segments();
    cleanup_old_waveforms();

    log_info!("Startup cleanup completed");

    tauri::Builder::default()
        .plugin(tauri_plugin_libmpv::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(prevent_default())
        .manage(
            ProcessManager::new().expect("Failed to create Windows Job Object. This application requires Windows Vista or later.")
        )
        .manage(WaveformJobRegistry::new())
        .invoke_handler(tauri::generate_handler![
            commands::export::export_segments,
            commands::metadata::get_video_metadata,
            stream_waveform,
            cancel_waveform,
            commands::config::set_app_config,
            commands::config::set_app_config_var,
            cancel_all_tasks,
            commands::session::set_session,
        ])
        .setup(|app| {
        		log_debug!("Loading config and session for frontend injection");

            let config = config::load_config(app.app_handle()).unwrap_or_default();
            let config_json = serde_json::to_string(&config).unwrap_or_default();

            let session = load_session(app.app_handle());
            let session_json = serde_json::to_string(&session).unwrap_or_default();

            let _window = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("VideoCut")
                .inner_size(1000.0, 700.0)
                .min_inner_size(800.0, 550.0)
                .drag_and_drop(true)
                .transparent(true)
                .visible(false)
                .initialization_script(&format!("window.__CONFIG__={};window.__SESSION__={};", config_json, session_json))
                .build();

            #[cfg(debug_assertions)]
            _window?.open_devtools();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn cancel_all_tasks(
    manager: tauri::State<'_, ProcessManager>,
    registry: tauri::State<'_, WaveformJobRegistry>,
) -> Result<(), String> {
    log_warn!("User requested cancellation of all tasks");

    registry.cancel_all();
    manager.kill_all();
    Ok(())
}
