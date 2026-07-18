use crate::error::{AppError, Result};
use crate::logger::{log_debug, log_error};
use std::path::PathBuf;
use tauri::Manager;

pub fn get_ffmpeg_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    if let Ok(resource_path) = app_handle
        .path()
        .resolve("lib/ffmpeg.exe", tauri::path::BaseDirectory::Resource)
    {
        if resource_path.exists() {
            log_debug!(path = ?resource_path, "Resolved FFmpeg binary path");

            return Ok(resource_path);
        }
    }

    #[cfg(debug_assertions)]
    {
        let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("debug")
            .join("ffmpeg.exe");
        if dev_path.exists() {
            log_debug!(path = ?dev_path, "Resolved FFmpeg binary path");

            return Ok(dev_path);
        }
    }

    log_error!("FFmpeg binary not found in resource or debug paths");

    Err(AppError::FFmpegError("FFmpeg binary not found".to_string()))
}

pub fn get_ffprobe_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    if let Ok(resource_path) = app_handle
        .path()
        .resolve("lib/ffprobe.exe", tauri::path::BaseDirectory::Resource)
    {
        if resource_path.exists() {
            log_debug!(path = ?resource_path, "Resolved FFprobe binary path");

            return Ok(resource_path);
        }
    }

    #[cfg(debug_assertions)]
    {
        let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("debug")
            .join("ffprobe.exe");
        if dev_path.exists() {
            log_debug!(path = ?dev_path, "Resolved FFprobe binary path");

            return Ok(dev_path);
        }
    }

    log_error!("FFprobe binary not found in resource or debug paths");

    Err(AppError::FFprobeError(
        "FFprobe binary not found".to_string(),
    ))
}

pub fn app_temp_dir() -> PathBuf {
    std::env::temp_dir().join("io.github.constdefe.tauri-video-cut")
}
