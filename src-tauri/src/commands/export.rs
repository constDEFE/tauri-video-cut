use crate::core::ProcessManager;
use crate::core::ffmpeg::{executor, keyframes, probe, smart_cut};
use crate::error::{AppError, Result};
use crate::logger::{log_debug, log_info, log_warn};
use crate::session::blank_session;
use crate::types::export::{ExportProgress, ExportRequest, ExportResult};
use crate::utils::paths::{get_ffmpeg_path, get_ffprobe_path};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tracing::instrument;

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

    let metadata = probe::probe_video(
        &ffprobe_path,
        &ffmpeg_path,
        &request.video_path,
        process_manager.inner(),
    )
    .await?;

    let keyframes =
        probe::get_keyframes(&ffprobe_path, &request.video_path, process_manager.inner()).await?;

    let total_segments = request.segments.len();
    let mut output_files = Vec::new();
    let ext = executor::get_output_extension(&request.video_path);

    let process_manager = Arc::new((*process_manager).clone());

    let mut segment_times: Vec<f64> = Vec::new();

    for (idx, segment) in request.segments.iter().enumerate() {
        let segment_start = Instant::now();
        let mut ema_eta: f64 = 0.0; // exponential moving average for ETA

        let output_filename = format!("{}-{:03}.{}", request.file_prefix, idx + 1, ext);
        let output_path = Path::new(&request.output_folder).join(&output_filename);

        let audio_stream_indices: Vec<usize> = segment
            .audio_tracks
            .iter()
            .filter_map(|&track_idx| {
                if track_idx == 0 {
                    return None;
                }
                let array_idx = track_idx - 1;
                metadata.audio_tracks.get(array_idx)?;
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
                let array_idx = track_idx - 1;
                metadata.audio_tracks.get(array_idx)
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
                let k2 =
                    keyframes::find_next_keyframe(segment.start, &keyframes).ok_or_else(|| {
                        AppError::ExportError(format!(
                            "Cannot find keyframe after segment {} start",
                            idx + 1
                        ))
                    })?;

                let k3 =
                    keyframes::find_prev_keyframe(segment.end, &keyframes).ok_or_else(|| {
                        AppError::ExportError(format!(
                            "Cannot find keyframe before segment {} end",
                            idx + 1
                        ))
                    })?;

                executor::CutMode::SmartCut {
                    k1,
                    k2,
                    k3,
                    k4,
                    start_is_keyframe,
                    end_is_keyframe,
                }
            }
        } else {
            executor::CutMode::StreamCopy
        };

        log_info!(segment = idx + 1, total = total_segments, start = segment.start, end = segment.end, cut_mode = ?cut_mode, "Processing segment");

        let app_handle_clone = app_handle.clone();
        let current_segment = idx + 1;
        let segment_times_len = segment_times.len();
        let segment_times_avg = if !segment_times.is_empty() {
            segment_times.iter().sum::<f64>() / segment_times.len() as f64
        } else {
            0.0
        };

        let export_duration = k4 - k1;

        match cut_mode {
            executor::CutMode::StreamCopy => {
                log_debug!(segment = idx + 1, "Using stream copy mode");

                let args = executor::build_export_args(
                    &request.video_path,
                    output_path.to_str().unwrap(),
                    k1,
                    k4,
                    &audio_stream_indices,
                    &selected_audio_tracks,
                );

                executor::execute_ffmpeg_with_progress(
                    &ffmpeg_path,
                    &args,
                    export_duration,
                    &mut move |progress| {
                        // Use EMA for stable ETA; avoid division by near-zero progress
                        if segment_times_len == 0 && progress > 1.0 {
                            ema_eta = segment_start.elapsed().as_secs_f64() * 100.0 / progress;
                        } else if progress > 1.0 {
                            ema_eta = 0.9 * ema_eta
                                + 0.1 * (segment_start.elapsed().as_secs_f64() * 100.0 / progress);
                        }
                        let avg_time_per_segment = if segment_times_len > 0 {
                            segment_times_avg
                        } else if ema_eta > 0.0 {
                            ema_eta
                        } else {
                            0.0
                        };

                        let current_segment_remaining =
                            (100.0 - progress).max(0.0) / 100.0 * avg_time_per_segment;
                        let remaining_segments = (total_segments - current_segment) as f64;
                        let eta = current_segment_remaining.max(0.0)
                            + (remaining_segments * avg_time_per_segment);

                        let _ = app_handle_clone.emit(
                            "export-progress",
                            ExportProgress {
                                current_segment,
                                total_segments,
                                current_segment_progress: progress,
                                eta_seconds: eta,
                            },
                        );
                    },
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

                smart_cut::execute_smart_cut(
                    &ffmpeg_path,
                    &request.video_path,
                    output_path.to_str().unwrap(),
                    k1,
                    k2,
                    k3,
                    k4,
                    segment.start,
                    segment.end,
                    start_is_keyframe,
                    end_is_keyframe,
                    &audio_stream_indices,
                    &selected_audio_tracks,
                    &metadata.video_codec,
                    &process_manager.clone(),
                    move |progress| {
                        // Use EMA for stable ETA; avoid division by near-zero progress
                        if segment_times_len == 0 && progress > 1.0 {
                            ema_eta = segment_start.elapsed().as_secs_f64() * 100.0 / progress;
                        } else if progress > 1.0 {
                            ema_eta = 0.9 * ema_eta
                                + 0.1 * (segment_start.elapsed().as_secs_f64() * 100.0 / progress);
                        }
                        let avg_time_per_segment = if segment_times_len > 0 {
                            segment_times_avg
                        } else if ema_eta > 0.0 {
                            ema_eta
                        } else {
                            0.0
                        };

                        let current_segment_remaining =
                            (100.0 - progress).max(0.0) / 100.0 * avg_time_per_segment;
                        let remaining_segments = (total_segments - current_segment) as f64;
                        let eta = current_segment_remaining.max(0.0)
                            + (remaining_segments * avg_time_per_segment);

                        let _ = app_handle_clone.emit(
                            "export-progress",
                            ExportProgress {
                                current_segment,
                                total_segments,
                                current_segment_progress: progress,
                                eta_seconds: eta,
                            },
                        );
                    },
                )
                .await?;
            }
        }

        segment_times.push(segment_start.elapsed().as_secs_f64());
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
