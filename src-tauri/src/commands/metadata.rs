use crate::core::ffmpeg::probe;
use crate::utils::paths::get_ffprobe_path;
use crate::core::process::ProcessManager;
use crate::error::Result;
use crate::logger;
use crate::types::metadata::VideoMetadata;

#[tauri::command]
pub async fn get_video_metadata(
    app_handle: tauri::AppHandle,
    video_path: String,
    process_manager: tauri::State<'_, ProcessManager>,
) -> Result<VideoMetadata> {
    logger::log_info(&format!("Fetching metadata for: {}", video_path));
    let ffprobe_path = get_ffprobe_path(&app_handle)?;
    let metadata = probe::probe_video(&ffprobe_path, &video_path, process_manager.inner()).await?;
    logger::log_info(&format!("Metadata fetched: duration={:.2}s, {}x{}", metadata.duration, metadata.width, metadata.height));
    Ok(metadata)
}
