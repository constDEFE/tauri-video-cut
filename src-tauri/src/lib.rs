mod commands;
mod config;
mod core;
mod error;
mod logger;
mod models;
mod session;
mod types;
mod utils;

use commands::{export, metadata};
use core::process::ProcessManager;
use models::AppConfig;
use session::{load_session, save_session};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

fn cleanup_orphaned_temp_segments() {
    let temp_dir = std::env::temp_dir()
        .join("io.github.constdefe.tauri-video-cut")
        .join("temp_segments");

    if !temp_dir.exists() {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }
}

fn add_lib_to_dll_search_path() {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let lib_dir = exe_dir.join("lib");
            if lib_dir.exists() {
                let wide: Vec<u16> = lib_dir.as_os_str().encode_wide().chain(once(0)).collect();
                unsafe extern "system" {
                    fn SetDllDirectoryW(lpPathName: *const u16) -> i32;
                }
                unsafe {
                    SetDllDirectoryW(wide.as_ptr());
                }
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logger::init();
    add_lib_to_dll_search_path();
    cleanup_orphaned_temp_segments();

    tauri::Builder::default()
        .plugin(tauri_plugin_libmpv::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(
            ProcessManager::new().expect("Failed to create Windows Job Object. This application requires Windows Vista or later.")
        )
        .invoke_handler(tauri::generate_handler![
            metadata::get_video_metadata,
            export::export_segments,
            set_app_config,
            set_app_config_var,
            cancel_all_tasks,
            set_session,
        ])
        .setup(|app| {
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
async fn cancel_all_tasks(manager: tauri::State<'_, ProcessManager>) -> Result<(), String> {
    manager.kill_all();
    Ok(())
}

#[tauri::command]
async fn set_app_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    config::set_app_config(&app, &config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_app_config_var(
    app: tauri::AppHandle,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    config::set_app_config_var(&app, &key, value)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_session(app: tauri::AppHandle, session: session::Session) -> Result<(), String> {
    save_session(&app, &session)
        .await
        .map_err(|e| e.to_string())
}
