use crate::error::{AppError, Result};
use crate::ffmpeg::{executor, get_ffmpeg_path, get_ffprobe_path, keyframes, probe};
use crate::models::{ExportProgress, ExportRequest, ExportResult};
use crate::logger;
use std::path::Path;
use std::time::Instant;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn export_segments(
    app_handle: AppHandle,
    request: ExportRequest,
) -> Result<ExportResult> {
    logger::log_info(&format!("Export started: {} segments from {}", request.segments.len(), request.video_path));
    
    if !Path::new(&request.video_path).exists() {
        logger::log_error(&format!("Video file not found: {}", request.video_path));
        return Err(AppError::FileNotFound(request.video_path.clone()));
    }

    if !Path::new(&request.output_folder).exists() {
        logger::log_error(&format!("Output folder not found: {}", request.output_folder));
        return Err(AppError::ExportError(
            "Output folder does not exist".to_string(),
        ));
    }

    if request.segments.is_empty() {
        logger::log_warn("No segments provided for export");
        return Err(AppError::InvalidSegment(
            "No segments to export".to_string(),
        ));
    }

    let ffmpeg_path = get_ffmpeg_path(&app_handle)?;
    let ffprobe_path = get_ffprobe_path(&app_handle)?;
    let metadata = probe::probe_video(&ffprobe_path, &request.video_path)?;

    let keyframes = probe::get_keyframes(&ffprobe_path, &request.video_path)?;

    let total_segments = request.segments.len();
    let mut output_files = Vec::new();
    let ext = executor::get_output_extension(&request.video_path);

    let mut segment_times: Vec<f64> = Vec::new();

    for (idx, segment) in request.segments.iter().enumerate() {
        let segment_start = Instant::now();

        let output_filename = format!("{}-{:03}.{}", request.file_prefix, idx + 1, ext);
        let output_path = Path::new(&request.output_folder).join(&output_filename);

        let audio_stream_indices: Vec<usize> = segment
            .audio_tracks
            .iter()
            .filter_map(|&track_idx| {
                // track_idx is 1-based, convert to 0-based array index
                if track_idx == 0 {
                    return None;
                }
                let array_idx = track_idx - 1;
                let track = metadata.audio_tracks.get(array_idx)?;
                Some(track.index)
            })
            .collect();

        let start_is_keyframe = keyframes::is_keyframe(segment.start, &keyframes);
        let end_is_keyframe = keyframes::is_keyframe(segment.end, &keyframes);

        let k1 = if start_is_keyframe {
            segment.start
        } else {
            keyframes::find_prev_keyframe(segment.start, &keyframes)
                .unwrap_or(segment.start)
        };

        let k4 = if end_is_keyframe {
            segment.end
        } else {
            keyframes::find_next_keyframe(segment.end, &keyframes)
                .unwrap_or(segment.end)
        };

        let cut_mode = if request.smart_cut {
            if start_is_keyframe && end_is_keyframe {
                executor::CutMode::StreamCopy
            } else {
                let k2 = keyframes::find_next_keyframe(segment.start, &keyframes)
                    .ok_or_else(|| {
                        AppError::ExportError(format!(
                            "Cannot find keyframe after segment {} start",
                            idx + 1
                        ))
                    })?;

                let k3 = keyframes::find_prev_keyframe(segment.end, &keyframes)
                    .ok_or_else(|| {
                        AppError::ExportError(format!(
                            "Cannot find keyframe before segment {} end",
                            idx + 1
                        ))
                    })?;

                executor::CutMode::SmartCut {
                    k1, k2, k3, k4,
                    start_is_keyframe,
                    end_is_keyframe,
                }
            }
        } else {
            executor::CutMode::StreamCopy
        };

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
                let args = executor::build_export_args(
                    &request.video_path,
                    output_path.to_str().unwrap(),
                    k1,
                    k4,
                    &audio_stream_indices,
                );

                executor::execute_ffmpeg_with_progress(
                    &ffmpeg_path,
                    &args,
                    export_duration,
                    move |progress| {
                        let avg_time_per_segment = if segment_times_len > 0 {
                            segment_times_avg
                        } else {
                            segment_start.elapsed().as_secs_f64() / (progress / 100.0).max(0.01)
                        };

                        let current_segment_remaining =
                            (100.0 - progress) / 100.0 * avg_time_per_segment;
                        let remaining_segments = (total_segments - current_segment) as f64;
                        let eta =
                            current_segment_remaining + (remaining_segments * avg_time_per_segment);

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
                )?;
            }
            executor::CutMode::SmartCut { k1, k2, k3, k4, start_is_keyframe, end_is_keyframe } => {
                executor::execute_smart_cut(
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
                    &metadata.video_codec,
                    move |progress| {
                        let avg_time_per_segment = if segment_times_len > 0 {
                            segment_times_avg
                        } else {
                            segment_start.elapsed().as_secs_f64() / (progress / 100.0).max(0.01)
                        };

                        let current_segment_remaining =
                            (100.0 - progress) / 100.0 * avg_time_per_segment;
                        let remaining_segments = (total_segments - current_segment) as f64;
                        let eta =
                            current_segment_remaining + (remaining_segments * avg_time_per_segment);

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
                )?;
            }
        }

        segment_times.push(segment_start.elapsed().as_secs_f64());
        output_files.push(output_path.to_string_lossy().to_string());
        logger::log_info(&format!("Segment {}/{} exported successfully", idx + 1, total_segments));
    }

    logger::log_info(&format!("Export completed: {} files generated", output_files.len()));
    
    Ok(ExportResult {
        success: true,
        output_files,
    })
}
