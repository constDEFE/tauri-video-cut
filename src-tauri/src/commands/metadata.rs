use crate::error::Result;
use crate::ffmpeg::{get_ffprobe_path, probe};
use crate::models::VideoMetadata;
use crate::logger;

#[tauri::command]
pub async fn get_video_metadata(
    app_handle: tauri::AppHandle,
    video_path: String,
) -> Result<VideoMetadata> {
    logger::log_info(&format!("Fetching metadata for: {}", video_path));
    let ffprobe_path = get_ffprobe_path(&app_handle)?;
    let metadata = probe::probe_video(&ffprobe_path, &video_path)?;
    logger::log_info(&format!("Metadata fetched: duration={:.2}s, {}x{}", metadata.duration, metadata.width, metadata.height));
    Ok(metadata)
}
