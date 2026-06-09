mod commands;
mod config;
mod error;
mod ffmpeg;
mod logger;
mod models;

use commands::{export, metadata};
use models::AppConfig;
use tauri::{WebviewUrl, WebviewWindowBuilder};

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
    add_lib_to_dll_search_path();
    logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_libmpv::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let config = config::load_config().unwrap_or_default();
            let config_json = serde_json::to_string(&config).unwrap_or_default();

            let _window = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("VideoCut")
                .inner_size(1000.0, 700.0)
                .min_inner_size(800.0, 550.0)
                .drag_and_drop(true)
                .transparent(true)
                .visible(false)
                .initialization_script(&format!("window.__CONFIG__ = {};", config_json))
                .build();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            metadata::get_video_metadata,
            export::export_segments,
            set_app_config,
            set_app_config_var,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn set_app_config(config: AppConfig) -> Result<(), String> {
    config::set_app_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_app_config_var(key: String, value: serde_json::Value) -> Result<(), String> {
    config::set_app_config_var(&key, value).map_err(|e| e.to_string())
}
