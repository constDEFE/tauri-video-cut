use crate::core::ProcessManager;
use crate::core::ffmpeg::capabilities;
use crate::core::ffmpeg::executor::{
    add_audio_mappings_with_metadata, execute_ffmpeg_with_progress, get_output_extension,
};
use crate::error::{AppError, Result};
use crate::logger::{log_debug, log_error, log_info, log_warn};
use crate::types::metadata::AudioTrack;
use crate::utils::cmd::new_command;
use crate::utils::paths::app_temp_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;

const DEFAULT_X264_QP: u32 = 18;
const DEFAULT_X265_QP: u32 = 18;
const DEFAULT_SVT_AV1_QP: u32 = 20;
const DEFAULT_VP9_CRF: u32 = 18;
const DEFAULT_QSCALE: u32 = 2;

const DEFAULT_X264_PRESET: &str = "medium";
const DEFAULT_SVT_AV1_PRESET: &str = "6";

/// RAII guard that cleans up all temp files on drop (success or panic).
pub struct TempCleanup {
    paths: Vec<PathBuf>,
}

impl TempCleanup {
    pub fn new(paths: Vec<PathBuf>) -> Self {
        Self { paths }
    }
}

impl Drop for TempCleanup {
    fn drop(&mut self) {
        for p in &self.paths {
            let _ = fs::remove_file(p);
        }
    }
}

pub async fn execute_smart_cut<F>(
    ffmpeg_path: &Path,
    input_path: &str,
    output_path: &str,
    k1: f64,
    k2: f64,
    k3: f64,
    k4: f64,
    start: f64,
    end: f64,
    start_is_keyframe: bool,
    end_is_keyframe: bool,
    audio_stream_indices: &[usize],
    audio_tracks: &[&AudioTrack],
    video_codec: &str,
    process_manager: &Arc<ProcessManager>,
    mut progress_callback: F,
) -> Result<()>
where
    F: FnMut(f64) + Send + 'static + Clone,
{
    log_info!(input = %input_path, k1, k2, k3, k4, start_is_keyframe, end_is_keyframe, codec = %video_codec, "Starting smart cut");

    let caps = capabilities::get_hw_capabilities(ffmpeg_path, process_manager.as_ref()).await?;
    let encoder_chain = capabilities::get_encoder_fallback_chain(video_codec, &caps);

    let temp_dir = app_temp_dir().join("temp_segments");
    fs::create_dir_all(&temp_dir).map_err(|e| {
        log_error!(path = ?temp_dir, error = %e, "Failed to create temp segments directory");

        AppError::ExportError(format!("Failed to create temp dir: {}", e))
    })?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let ext = get_output_extension(input_path);
    let temp_start_encode = temp_dir.join(format!("start_encode_{}.{}", timestamp, ext));
    let temp_middle_copy = temp_dir.join(format!("middle_copy_{}.{}", timestamp, ext));
    let temp_end_encode = temp_dir.join(format!("end_encode_{}.{}", timestamp, ext));
    let temp_concat = temp_dir.join(format!("concat_{}.{}", timestamp, ext));

    let _cleanup = TempCleanup::new(vec![
        temp_start_encode.clone(),
        temp_middle_copy.clone(),
        temp_end_encode.clone(),
        temp_concat.clone(),
    ]);

    let encoding_weight = 10.0;
    let total_work = if !start_is_keyframe { k2 - k1 } else { 0.0 }
        + (k3 - k2)
        + if !end_is_keyframe { k4 - k3 } else { 0.0 };

    let start_work = if !start_is_keyframe { k2 - k1 } else { 0.0 };
    let middle_work = k3 - k2;
    let end_work = if !end_is_keyframe { k4 - k3 } else { 0.0 };

    let start_weight = if !start_is_keyframe {
        start_work * encoding_weight / total_work
    } else {
        0.0
    };
    let middle_weight = middle_work / total_work;
    let end_weight = if !end_is_keyframe {
        end_work * encoding_weight / total_work
    } else {
        0.0
    };

    let mut parts = Vec::new();
    let mut current_progress = 0.0;
    let mut working_encoder: Option<String> = None;

    log_debug!(
        phase = "start_encode",
        duration = k2 - k1,
        "Encoding start segment"
    );

    if !start_is_keyframe {
        let duration = k2 - k1;
        let mut cb = progress_callback.clone();

        let encoder_used = encode_segment_with_fallback(
            ffmpeg_path,
            input_path,
            &temp_start_encode,
            k1,
            duration,
            &encoder_chain,
            audio_stream_indices,
            audio_tracks,
            Some(format!("expr:gte(t,{:.3})", k2 - k1)),
            move |prog| cb(current_progress + prog * start_weight),
            process_manager,
        )
        .await?;

        working_encoder = Some(encoder_used);
        parts.push(temp_start_encode.to_str().unwrap().to_string());
        current_progress += start_weight;
    }

    let copy_start = if start_is_keyframe { start } else { k2 };
    let copy_end = if end_is_keyframe { end } else { k3 };
    let copy_duration = copy_end - copy_start;

    let mut args_copy = vec![
        "-ss".to_string(),
        format!("{:.3}", copy_start),
        "-i".to_string(),
        input_path.to_string(),
        "-t".to_string(),
        format!("{:.3}", copy_duration),
        "-map".to_string(),
        "0:v:0".to_string(),
    ];

    add_audio_mappings_with_metadata(&mut args_copy, audio_stream_indices, audio_tracks, false);

    args_copy.extend([
        "-c".to_string(),
        "copy".to_string(),
        "-y".to_string(),
        "-progress".to_string(),
        "pipe:2".to_string(),
        temp_middle_copy.to_str().unwrap().to_string(),
    ]);

    log_debug!(
        phase = "middle_copy",
        copy_start,
        copy_end,
        duration = copy_duration,
        "Copying middle segment"
    );

    execute_ffmpeg_with_progress(
        ffmpeg_path,
        &args_copy,
        copy_duration,
        &mut |prog| {
            progress_callback(current_progress + prog * middle_weight);
        },
        process_manager,
    )
    .await?;

    parts.push(temp_middle_copy.to_str().unwrap().to_string());
    current_progress += middle_weight;

    log_debug!(
        phase = "end_encode",
        duration = k4 - k3,
        "Encoding end segment"
    );

    if !end_is_keyframe {
        let duration = k4 - k3;

        let chain_to_use = if let Some(ref encoder) = working_encoder {
            vec![encoder.clone()]
        } else {
            encoder_chain.clone()
        };

        let mut cb = progress_callback.clone();
        let encoder_used = encode_segment_with_fallback(
            ffmpeg_path,
            input_path,
            &temp_end_encode,
            k3,
            duration,
            &chain_to_use,
            audio_stream_indices,
            audio_tracks,
            Some("expr:eq(n,0)".to_string()),
            move |prog| cb(current_progress + prog * end_weight),
            process_manager,
        )
        .await?;

        if working_encoder.is_none() {
            working_encoder = Some(encoder_used);
        }

        parts.push(temp_end_encode.to_str().unwrap().to_string());
    }

    let concat_content = parts
        .iter()
        .map(|p| format!("file '{}'", p.replace('\\', "/")))
        .collect::<Vec<_>>()
        .join("\n");

    let concat_list = temp_dir.join(format!("concat_list_{}.txt", timestamp));
    fs::write(&concat_list, &concat_content)
        .map_err(|e| AppError::ExportError(format!("Failed to write concat list: {}", e)))?;

    let mut args_concat = vec![
        "-f".to_string(),
        "concat".to_string(),
        "-safe".to_string(),
        "0".to_string(),
        "-fflags".to_string(),
        "+genpts".to_string(),
        "-i".to_string(),
        concat_list.to_str().unwrap().to_string(),
        "-map".to_string(),
        "0:v".to_string(),
    ];

    add_audio_mappings_with_metadata(&mut args_concat, audio_stream_indices, audio_tracks, true);

    args_concat.extend([
        "-c".to_string(),
        "copy".to_string(),
        "-y".to_string(),
        temp_concat.to_str().unwrap().to_string(),
    ]);

    log_debug!(
        phase = "concat",
        parts = parts.len(),
        "Concatenating segments"
    );

    let mut child = new_command(ffmpeg_path)
        .args(&args_concat)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::FFmpegError(format!("Failed to spawn ffmpeg concat: {}", e)))?;

    process_manager.attach(&child)?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::FFmpegError("Failed to capture concat stderr".to_string()))?;

    let reader = tokio::io::BufReader::new(stderr);
    let lines = reader.lines();
    let mut lines = Box::pin(lines);

    let mut concat_log = Vec::new();
    while let Ok(Some(line)) = lines.next_line().await {
        concat_log.push(line);
    }

    let status = child
        .wait()
        .await
        .map_err(|e| AppError::FFmpegError(format!("Failed to wait for concat: {}", e)))?;

    if !status.success() {
        let log_output = concat_log.join("\n");

        log_error!(status = %status, stderr = %log_output, "Smart cut concatenation failed");

        return Err(AppError::FFmpegError(format!(
            "Concat failed with status: {}\n=== FFmpeg stderr ===\n{}",
            status, log_output
        )));
    }

    let concat_weight = 0.02;
    let trim_weight = 0.03;

    progress_callback(current_progress + concat_weight);

    let trim_start = if start_is_keyframe { 0.0 } else { start - k1 };
    let trim_duration = end - start;

    let mut args_trim = vec![
        "-ss".to_string(),
        format!("{:.3}", trim_start),
        "-i".to_string(),
        temp_concat.to_str().unwrap().to_string(),
        "-t".to_string(),
        format!("{:.3}", trim_duration),
        "-map".to_string(),
        "0:v:0".to_string(),
    ];

    add_audio_mappings_with_metadata(&mut args_trim, audio_stream_indices, audio_tracks, true);

    args_trim.extend([
        "-c".to_string(),
        "copy".to_string(),
        "-y".to_string(),
        output_path.to_string(),
    ]);

    log_debug!(
        phase = "trim",
        trim_start,
        trim_duration,
        "Trimming concatenated output"
    );

    let mut trim_child = new_command(ffmpeg_path)
        .args(&args_trim)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::FFmpegError(format!("Failed to spawn ffmpeg trim: {}", e)))?;

    process_manager.attach(&trim_child)?;

    let trim_status = trim_child
        .wait()
        .await
        .map_err(|e| AppError::FFmpegError(format!("Failed to wait for trim: {}", e)))?;

    if !trim_status.success() {
        log_error!(status = %trim_status, "Smart cut final trim failed");

        return Err(AppError::FFmpegError(format!(
            "Trim failed with status: {}",
            trim_status
        )));
    }

    progress_callback(current_progress + concat_weight + trim_weight);

    let _ = fs::remove_file(&concat_list);

    Ok(())
}

async fn encode_segment_with_fallback<F>(
    ffmpeg_path: &Path,
    input_path: &str,
    output_path: &Path,
    start_time: f64,
    duration: f64,
    encoder_chain: &[String],
    audio_stream_indices: &[usize],
    audio_tracks: &[&AudioTrack],
    force_keyframes: Option<String>,
    mut progress_callback: F,
    process_manager: &Arc<ProcessManager>,
) -> Result<String>
where
    F: FnMut(f64) + Send + 'static,
{
    let mut last_error = None;

    for encoder in encoder_chain {
        log_debug!(encoder = %encoder, start_time, duration, "Trying encoder");

        let mut args = vec![
            "-ss".to_string(),
            format!("{:.3}", start_time),
            "-i".to_string(),
            input_path.to_string(),
            "-t".to_string(),
            format!("{:.3}", duration),
            "-c:v".to_string(),
            encoder.clone(),
        ];
        add_encoder_params(&mut args, encoder);
        if let Some(ref keyframes) = force_keyframes {
            args.extend(["-force_key_frames".to_string(), keyframes.clone()]);
        }
        args.extend([
            "-c:a".to_string(),
            "copy".to_string(),
            "-map".to_string(),
            "0:v:0".to_string(),
        ]);
        add_audio_mappings_with_metadata(&mut args, audio_stream_indices, audio_tracks, false);
        args.extend([
            "-y".to_string(),
            "-progress".to_string(),
            "pipe:2".to_string(),
            output_path.to_str().unwrap().to_string(),
        ]);

        match execute_ffmpeg_with_progress(
            ffmpeg_path,
            &args,
            duration,
            &mut progress_callback,
            process_manager,
        )
        .await
        {
            Ok(_) => return Ok(encoder.clone()),
            Err(e) => {
                log_warn!(encoder = %encoder, error = %e, "Encoder failed; trying next in chain");

                last_error = Some(e);
                let _ = fs::remove_file(output_path);
            }
        }
    }

    log_error!("All encoders in fallback chain exhausted");

    Err(last_error
        .unwrap_or_else(|| AppError::FFmpegError("Encoder fallback chain is empty".to_string())))
}

fn add_encoder_params(args: &mut Vec<String>, encoder: &str) {
    match encoder {
        "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" => args.extend([
            "-preset".to_string(),
            "p4".to_string(),
            "-rc".to_string(),
            "vbr".to_string(),
            "-cq".to_string(),
            DEFAULT_X264_QP.to_string(),
        ]),
        "libx264" | "libx265" => args.extend([
            "-preset".to_string(),
            DEFAULT_X264_PRESET.to_string(),
            "-qp".to_string(),
            if encoder == "libx264" {
                DEFAULT_X264_QP.to_string()
            } else {
                DEFAULT_X265_QP.to_string()
            },
        ]),
        "libsvtav1" => args.extend([
            "-preset".to_string(),
            DEFAULT_SVT_AV1_PRESET.to_string(),
            "-crf".to_string(),
            DEFAULT_SVT_AV1_QP.to_string(),
        ]),
        "libvpx-vp9" => args.extend([
            "-crf".to_string(),
            DEFAULT_VP9_CRF.to_string(),
            "-b:v".to_string(),
            "0".to_string(),
        ]),
        _ => args.extend(["-qscale:v".to_string(), DEFAULT_QSCALE.to_string()]),
    }
}
