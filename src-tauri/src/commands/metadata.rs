use tracing::instrument;

use crate::core::ProcessManager;
use crate::core::ffmpeg::probe;
use crate::error::Result;
use crate::logger::log_info;
use crate::types::metadata::VideoMetadata;
use crate::utils::paths::{get_ffmpeg_path, get_ffprobe_path};

#[instrument(skip(app_handle, process_manager), fields(video = %video_path), err(Debug))]
#[tauri::command]
pub async fn get_video_metadata(
    app_handle: tauri::AppHandle,
    video_path: String,
    process_manager: tauri::State<'_, ProcessManager>,
) -> Result<VideoMetadata> {
    log_info!("Fetching video metadata");

    let ffprobe_path = get_ffprobe_path(&app_handle)?;
    let ffmpeg_path = get_ffmpeg_path(&app_handle)?;

    let metadata = probe::probe_video(
        &ffprobe_path,
        &ffmpeg_path,
        &video_path,
        process_manager.inner(),
    )
    .await?;

    log_info!(
        duration = metadata.duration,
        width = metadata.width,
        height = metadata.height,
        has_waveforms = metadata.waveforms.is_some(),
        "Metadata fetched"
    );

    Ok(metadata)
}
