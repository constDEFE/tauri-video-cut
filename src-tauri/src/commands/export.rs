use crate::core::ProcessManager;
use crate::core::ffmpeg::{executor, keyframes, probe, smart_cut};
use crate::error::{AppError, Result};
use crate::logger::{log_debug, log_info, log_warn};
use crate::session::blank_session;
use crate::types::export::{ExportProgress, ExportRequest, ExportResult};
use crate::utils::paths::{get_ffmpeg_path, get_ffprobe_path};
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tracing::instrument;

fn create_progress_callback(
    app_handle: AppHandle,
    current_segment: usize,
    total_segments: usize,
) -> impl FnMut(f64) + Send + Clone + 'static {
    move |progress: f64| {
        let progress = progress.clamp(0.0, 100.0);
        let _ = app_handle.emit(
            "export-progress",
            ExportProgress {
                current_segment,
                total_segments,
                current_segment_progress: progress,
            },
        );
    }
}

#[instrument(skip(app_handle, process_manager), fields(segments = request.segments.len(), video = %request.video_path, smart_cut = request.smart_cut), err(Debug))]
#[tauri::command]
pub async fn export_segments(
    app_handle: AppHandle,
    request: ExportRequest,
    process_manager: tauri::State<'_, ProcessManager>,
) -> Result<ExportResult> {
    log_info!("Export started");

    if !Path::new(&request.video_path).exists() {
        return Err(AppError::FileNotFound(request.video_path.clone()));
    }

    if !Path::new(&request.output_folder).exists() {
        return Err(AppError::ExportError(
            "Output folder does not exist".to_string(),
        ));
    }

    if request.segments.is_empty() {
        return Err(AppError::InvalidSegment(
            "No segments to export".to_string(),
        ));
    }

    let ffmpeg_path = get_ffmpeg_path(&app_handle)?;
    let ffprobe_path = get_ffprobe_path(&app_handle)?;
    let (metadata, keyframes, source_params) = tokio::try_join!(
        probe::probe_video(
            &ffprobe_path,
            &ffmpeg_path,
            &request.video_path,
            process_manager.inner()
        ),
        probe::get_keyframes(&ffprobe_path, &request.video_path, process_manager.inner()),
        probe::probe_video_codec_params(
            &ffprobe_path,
            &request.video_path,
            process_manager.inner()
        ),
    )?;

    let total_segments = request.segments.len();
    let mut output_files = Vec::new();
    let ext = executor::get_output_extension(&request.video_path);

    let process_manager = Arc::new((*process_manager).clone());

    for (idx, segment) in request.segments.iter().enumerate() {
        let output_filename = format!("{}-{:03}.{}", request.file_prefix, idx + 1, ext);
        let output_path = Path::new(&request.output_folder).join(&output_filename);

        let audio_stream_indices: Vec<usize> = segment
            .audio_tracks
            .iter()
            .filter_map(|&track_idx| {
                if track_idx == 0 {
                    return None;
                }
                metadata.audio_tracks.get(track_idx - 1)?;
                Some(track_idx)
            })
            .collect();

        let selected_audio_tracks: Vec<&crate::types::metadata::AudioTrack> = segment
            .audio_tracks
            .iter()
            .filter_map(|&track_idx| {
                if track_idx == 0 {
                    return None;
                }
                metadata.audio_tracks.get(track_idx - 1)
            })
            .collect();

        let start_is_keyframe = keyframes::is_keyframe(segment.start, &keyframes);
        let end_is_keyframe = keyframes::is_keyframe(segment.end, &keyframes);

        let k1 = if start_is_keyframe {
            segment.start
        } else {
            keyframes::find_prev_keyframe(segment.start, &keyframes).unwrap_or(segment.start)
        };

        let k4 = if end_is_keyframe {
            segment.end
        } else {
            keyframes::find_next_keyframe(segment.end, &keyframes).unwrap_or(segment.end)
        };

        let cut_mode = if request.smart_cut {
            if start_is_keyframe && end_is_keyframe {
                executor::CutMode::StreamCopy
            } else {
                let k2 = keyframes::find_next_keyframe(segment.start, &keyframes);
                let k3 = keyframes::find_prev_keyframe(segment.end, &keyframes);
                match (k2, k3) {
                    (Some(k2), Some(k3)) if k2 < k3 => executor::CutMode::SmartCut {
                        k1,
                        k2,
                        k3,
                        k4,
                        start_is_keyframe,
                        end_is_keyframe,
                    },
                    _ => {
                        log_debug!(
                            segment = idx + 1,
                            k2 = ?k2,
                            k3 = ?k3,
                            "No valid intermediate keyframes; using full encode"
                        );

                        executor::CutMode::FullEncode
                    }
                }
            }
        } else {
            executor::CutMode::StreamCopy
        };

        log_info!(
            segment = idx + 1,
            total = total_segments,
            start = segment.start,
            end = segment.end,
            cut_mode = ?cut_mode,
            "Processing segment"
        );

        let current_segment = idx + 1;

        match cut_mode {
            executor::CutMode::StreamCopy => {
                log_debug!(segment = idx + 1, "Using stream copy mode");

                let export_duration = k4 - k1;
                let args = executor::build_export_args(
                    &request.video_path,
                    output_path.to_str().unwrap(),
                    k1,
                    k4,
                    &audio_stream_indices,
                    &selected_audio_tracks,
                );
                let mut cb =
                    create_progress_callback(app_handle.clone(), current_segment, total_segments);
                executor::execute_ffmpeg_with_progress(
                    &ffmpeg_path,
                    &args,
                    export_duration,
                    &mut cb,
                    process_manager.as_ref(),
                )
                .await?;
            }
            executor::CutMode::SmartCut {
                k1,
                k2,
                k3,
                k4,
                start_is_keyframe,
                end_is_keyframe,
            } => {
                log_debug!(segment = idx + 1, k1, k2, k3, k4, "Using smart cut mode");

                let cb =
                    create_progress_callback(app_handle.clone(), current_segment, total_segments);

                smart_cut::execute_smart_cut(
                    &ffmpeg_path,
                    &request.video_path,
                    output_path.to_str().unwrap(),
                    k1,
                    k2,
                    k3,
                    segment.start,
                    segment.end,
                    start_is_keyframe,
                    end_is_keyframe,
                    &audio_stream_indices,
                    &selected_audio_tracks,
                    &metadata.video_codec,
                    &source_params,
                    &process_manager.clone(),
                    cb,
                )
                .await?;
            }
            executor::CutMode::FullEncode => {
                log_debug!(segment = idx + 1, "Using full encode mode");

                let cb =
                    create_progress_callback(app_handle.clone(), current_segment, total_segments);
                smart_cut::execute_full_encode(
                    &ffmpeg_path,
                    &request.video_path,
                    output_path.to_str().unwrap(),
                    segment.start,
                    segment.end,
                    k1,
                    &audio_stream_indices,
                    &selected_audio_tracks,
                    &metadata.video_codec,
                    &process_manager.clone(),
                    cb,
                )
                .await?;
            }
        }

        output_files.push(output_path.to_string_lossy().to_string());

        log_info!(files = output_files.len(), "Export completed");
    }

    log_info!("Export completed: {} files generated", output_files.len());

    if let Err(e) = blank_session(&app_handle).await {
        log_warn!(error = %e, "Failed to blank session after export");
    }

    Ok(ExportResult {
        success: true,
        output_files,
    })
}
