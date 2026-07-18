use crate::core::ProcessManager;
use crate::error::{AppError, Result};
use crate::logger::log_error;
use crate::types::metadata::AudioTrack;
use crate::utils::cmd::new_command;
use regex::Regex;
use std::collections::VecDeque;
use std::path::Path;
use std::process::Stdio;
use std::sync::LazyLock;
use tokio::io::AsyncBufReadExt;

static TIME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"time=(\d{2}):(\d{2}):(\d{2}\.\d{2})").unwrap());

#[derive(Debug, Clone)]
pub enum CutMode {
    StreamCopy,
    SmartCut {
        k1: f64,
        k2: f64,
        k3: f64,
        k4: f64,
        start_is_keyframe: bool,
        end_is_keyframe: bool,
    },
}

pub fn add_audio_mappings_with_metadata(
    args: &mut Vec<String>,
    audio_stream_indices: &[usize],
    audio_tracks: &[&AudioTrack],
    is_generated_file: bool,
) {
    if audio_stream_indices.is_empty() {
        args.push("-an".to_string());
    } else {
        for (output_idx, &stream_idx) in audio_stream_indices.iter().enumerate() {
            args.push("-map".to_string());

            let idx_to_use = if is_generated_file {
                output_idx
            } else {
                stream_idx
            };

            if is_generated_file {
                args.push(format!("0:a:{}", idx_to_use));
            } else {
                args.push(format!("0:{}", idx_to_use));
            }

            if let Some(track) = audio_tracks.get(output_idx) {
                if let Some(name) = &track.name {
                    args.extend([
                        format!("-metadata:s:a:{}", output_idx),
                        format!("title={}", name),
                    ])
                }
            }
        }

        args.extend([
            "-movflags".to_string(),
            "+use_metadata_tags+faststart".to_string(),
            "-map_metadata".to_string(),
            "0".to_string(),
        ])
    }
}

pub async fn execute_ffmpeg_with_progress<'a, F>(
    ffmpeg_path: &Path,
    args: &[String],
    segment_duration: f64,
    progress_callback: &'a mut F,
    process_manager: &ProcessManager,
) -> Result<()>
where
    F: FnMut(f64) + Send,
{
    let mut child = new_command(ffmpeg_path)
        .args(args)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            log_error!(error = %e, args = ?args, "Failed to spawn ffmpeg");

            AppError::FFmpegError(format!("Failed to spawn ffmpeg: {}", e))
        })?;

    process_manager.attach(&child)?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::FFmpegError("Failed to capture stderr".to_string()))?;

    let reader = tokio::io::BufReader::new(stderr);

    let mut log_buffer: VecDeque<String> = VecDeque::with_capacity(100);

    let lines = reader.lines();
    let mut lines = Box::pin(lines);

    while let Ok(Some(result)) = lines.next_line().await {
        let line: String = result;
        if let Some(caps) = TIME_REGEX.captures(&line) {
            let hours: f64 = caps[1].parse().unwrap_or(0.0);
            let minutes: f64 = caps[2].parse().unwrap_or(0.0);
            let seconds: f64 = caps[3].parse().unwrap_or(0.0);

            let current_time = hours * 3600.0 + minutes * 60.0 + seconds;
            let progress = (current_time / segment_duration * 100.0).min(100.0);

            progress_callback(progress);
        }

        log_buffer.push_back(line);
        if log_buffer.len() > 100 {
            log_buffer.pop_front();
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| AppError::FFmpegError(format!("Failed to wait for ffmpeg: {}", e)))?;

    if !status.success() {
        let trailing_logs = log_buffer.into_iter().collect::<Vec<String>>().join("\n");

        log_error!(status = %status, stderr = %trailing_logs, "FFmpeg exited with non-zero status");

        return Err(AppError::FFmpegError(format!(
            "FFmpeg exited with status: {}\n--- FFmpeg Stderr Log ---\n{}",
            status,
            if trailing_logs.is_empty() {
                "[No log output captured]"
            } else {
                &trailing_logs
            }
        )));
    }

    Ok(())
}

pub fn build_export_args(
    input_path: &str,
    output_path: &str,
    start: f64,
    end: f64,
    audio_stream_indices: &[usize],
    audio_tracks: &[&AudioTrack],
) -> Vec<String> {
    let mut args = Vec::new();

    args.extend([
        "-ss".to_string(),
        format!("{:.3}", start),
        "-i".to_string(),
        input_path.to_string(),
        "-to".to_string(),
        format!("{:.3}", end - start),
        "-map".to_string(),
        "0:v:0".to_string(),
    ]);

    add_audio_mappings_with_metadata(&mut args, audio_stream_indices, audio_tracks, false);

    args.extend([
        "-c".to_string(),
        "copy".to_string(),
        "-y".to_string(),
        "-progress".to_string(),
        "pipe:2".to_string(),
        output_path.to_string(),
    ]);

    args
}

pub fn get_output_extension(input_path: &str) -> String {
    std::path::Path::new(input_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp4")
        .to_string()
}
