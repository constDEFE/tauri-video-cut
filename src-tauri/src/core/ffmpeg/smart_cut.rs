use crate::core::ProcessManager;
use crate::core::ffmpeg::executor::{
    add_audio_mappings_with_metadata, execute_ffmpeg_with_progress, get_output_extension,
};
use crate::core::ffmpeg::probe::VideoCodecParams;
use crate::error::{AppError, Result};
use crate::logger::{log_debug, log_error, log_info, log_warn};
use crate::types::metadata::AudioTrack;
use crate::utils::cmd::new_command;
use crate::utils::paths::app_temp_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

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

fn level_to_arg(level: u32) -> String {
    if level == 9 {
        "1b".to_string()
    } else {
        format!("{}.{}", level / 10, level % 10)
    }
}

pub async fn execute_full_encode<F>(
    ffmpeg_path: &Path,
    input_path: &str,
    output_path: &str,
    start: f64,
    end: f64,
    k1: f64,
    audio_stream_indices: &[usize],
    audio_tracks: &[&AudioTrack],
    video_codec: &str,
    process_manager: &Arc<ProcessManager>,
    progress_callback: F,
) -> Result<()>
where
    F: FnMut(f64) + Send + 'static,
{
    let duration = end - start;
    log_info!(
        input = %input_path, start, end, duration, k1, codec = %video_codec,
        "Starting full segment encode (no intermediate keyframes)"
    );
    let encoder = get_matching_encoder(video_codec);
    encode_segment(
        ffmpeg_path,
        input_path,
        Path::new(output_path),
        k1,
        Some(start - k1),
        duration,
        &encoder,
        audio_stream_indices,
        audio_tracks,
        None,
        true,
        None,
        progress_callback,
        process_manager,
    )
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
pub async fn execute_smart_cut<F>(
    ffmpeg_path: &Path,
    input_path: &str,
    output_path: &str,
    k1: f64,
    k2: f64,
    k3: f64,
    start: f64,
    end: f64,
    start_is_keyframe: bool,
    end_is_keyframe: bool,
    audio_stream_indices: &[usize],
    audio_tracks: &[&AudioTrack],
    video_codec: &str,
    source_params: &VideoCodecParams,
    process_manager: &Arc<ProcessManager>,
    mut progress_callback: F,
) -> Result<()>
where
    F: FnMut(f64) + Send + 'static + Clone,
{
    log_info!(
        input = %input_path, k1, k2, k3,
        start_is_keyframe, end_is_keyframe,
        codec = %video_codec,
        "Starting smart cut"
    );

    let encoder = get_matching_encoder(video_codec);

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
    let temp_video = temp_dir.join(format!("concat_video_{}.{}", timestamp, ext));

    let _cleanup = TempCleanup::new(vec![
        temp_start_encode.clone(),
        temp_middle_copy.clone(),
        temp_end_encode.clone(),
        temp_video.clone(),
    ]);

    let has_audio = !audio_stream_indices.is_empty();

    const ENCODING_WEIGHT: f64 = 10.0;
    const COPY_WEIGHT: f64 = 1.0;
    const AUDIO_WEIGHT: f64 = 1.0;

    let copy_start = if start_is_keyframe { start } else { k2 };
    let copy_end = if end_is_keyframe { end } else { k3 };

    let head_work = if !start_is_keyframe {
        (k2 - start) * ENCODING_WEIGHT
    } else {
        0.0
    };
    let middle_work = (copy_end - copy_start) * COPY_WEIGHT;
    let tail_work = if !end_is_keyframe {
        (end - k3) * ENCODING_WEIGHT
    } else {
        0.0
    };
    let concat_work = (end - start) * COPY_WEIGHT;
    let audio_work = if has_audio {
        (end - start) * AUDIO_WEIGHT
    } else {
        0.0
    };

    let total_work = head_work + middle_work + tail_work + concat_work + audio_work;
    let width = |w: f64| {
        if total_work > 0.0 {
            w / total_work * 100.0
        } else {
            0.0
        }
    };
    let head_width = width(head_work);
    let middle_width = width(middle_work);
    let tail_width = width(tail_work);
    let concat_width = width(concat_work);
    let audio_width = width(audio_work);

    let mut parts: Vec<String> = Vec::new();
    let mut current_progress: f64 = 0.0;

    if !start_is_keyframe {
        let head_duration = k2 - start;

        log_debug!(
            phase = "start_encode",
            input_seek = k1,
            output_seek = start - k1,
            duration = head_duration,
            "Encoding head segment (video only)"
        );

        let mut cb = progress_callback.clone();
        let base = current_progress;

        encode_segment(
            ffmpeg_path,
            input_path,
            &temp_start_encode,
            k1,
            Some(start - k1),
            head_duration,
            &encoder,
            audio_stream_indices,
            audio_tracks,
            None,
            false,
            Some(source_params),
            move |prog| cb(base + prog / 100.0 * head_width),
            process_manager,
        )
        .await?;
        parts.push(temp_start_encode.to_str().unwrap().to_string());
        current_progress += head_width;
    }

    let copy_duration = copy_end - copy_start;
    if copy_duration > 0.0 {
        log_debug!(
            phase = "middle_copy",
            copy_start,
            copy_end,
            duration = copy_duration,
            "Copying middle segment (video only)"
        );

        let args_copy = vec![
            "-hide_banner".to_string(),
            "-ss".to_string(),
            format!("{:.6}", copy_start),
            "-i".to_string(),
            input_path.to_string(),
            "-t".to_string(),
            format!("{:.6}", copy_duration),
            "-map".to_string(),
            "0:v:0".to_string(),
            "-an".to_string(),
            "-c".to_string(),
            "copy".to_string(),
            "-avoid_negative_ts".to_string(),
            "make_zero".to_string(),
            "-y".to_string(),
            "-progress".to_string(),
            "pipe:2".to_string(),
            temp_middle_copy.to_str().unwrap().to_string(),
        ];
        execute_ffmpeg_with_progress(
            ffmpeg_path,
            &args_copy,
            copy_duration,
            &mut |prog| {
                progress_callback(current_progress + prog / 100.0 * middle_width);
            },
            process_manager,
        )
        .await?;

        parts.push(temp_middle_copy.to_str().unwrap().to_string());
        current_progress += middle_width;
    }

    if !end_is_keyframe {
        let tail_duration = end - k3;
        log_debug!(
            phase = "end_encode",
            input_seek = k3,
            duration = tail_duration,
            "Encoding tail segment (video only)"
        );

        let mut cb = progress_callback.clone();
        let base = current_progress;

        encode_segment(
            ffmpeg_path,
            input_path,
            &temp_end_encode,
            k3,
            None,
            tail_duration,
            &encoder,
            audio_stream_indices,
            audio_tracks,
            Some("expr:eq(n,0)".to_string()),
            false,
            Some(source_params),
            move |prog| cb(base + prog / 100.0 * tail_width),
            process_manager,
        )
        .await?;

        parts.push(temp_end_encode.to_str().unwrap().to_string());
        current_progress += tail_width;
    }

    let concat_content = parts
        .iter()
        .map(|p| format!("file '{}'", p.replace('\\', "/")))
        .collect::<Vec<_>>()
        .join("\n");

    let concat_list = temp_dir.join(format!("concat_list_{}.txt", timestamp));

    fs::write(&concat_list, &concat_content)
        .map_err(|e| AppError::ExportError(format!("Failed to write concat list: {}", e)))?;

    let concat_target: PathBuf = if has_audio {
        temp_video.clone()
    } else {
        PathBuf::from(output_path)
    };

    let args_concat = vec![
        "-hide_banner".to_string(),
        "-f".to_string(),
        "concat".to_string(),
        "-safe".to_string(),
        "0".to_string(),
        "-i".to_string(),
        concat_list.to_str().unwrap().to_string(),
        "-map".to_string(),
        "0:v".to_string(),
        "-an".to_string(),
        "-c".to_string(),
        "copy".to_string(),
        "-y".to_string(),
        "-progress".to_string(),
        "pipe:2".to_string(),
        concat_target.to_str().unwrap().to_string(),
    ];

    log_debug!(
        phase = "concat",
        parts = parts.len(),
        "Concatenating video-only segments"
    );

    execute_ffmpeg_with_progress(
        ffmpeg_path,
        &args_concat,
        end - start,
        &mut |prog| {
            progress_callback(current_progress + prog / 100.0 * concat_width);
        },
        process_manager,
    )
    .await?;

    current_progress += concat_width;

    let mut boundaries: Vec<f64> = Vec::new();
    let mut t = 0.0;

    if !start_is_keyframe {
        t += k2 - start;
        boundaries.push(t);
    }

    t += copy_end - copy_start;

    if !end_is_keyframe {
        boundaries.push(t);
    }

    if !validate_boundaries(ffmpeg_path, &concat_target, &boundaries, process_manager).await {
        if !has_audio {
            let _ = fs::remove_file(output_path);
        }

        log_warn!(
            "Boundary decode validation failed: re-encoded parts are not parameter-compatible with this source. Falling back to full encode for this segment."
        );

        let base = current_progress;
        let remaining = (100.0 - base).max(0.0);
        let mut done_cb = progress_callback.clone();
        let result = execute_full_encode(
            ffmpeg_path,
            input_path,
            output_path,
            start,
            end,
            k1,
            audio_stream_indices,
            audio_tracks,
            video_codec,
            process_manager,
            move |prog| progress_callback(base + prog / 100.0 * remaining),
        )
        .await;

        if result.is_ok() {
            done_cb(100.0);
        }

        return result;
    }

    if has_audio {
        let mut args_mux = vec![
            "-hide_banner".to_string(),
            "-ss".to_string(),
            format!("{:.6}", start),
            "-i".to_string(),
            input_path.to_string(),
            "-i".to_string(),
            temp_video.to_str().unwrap().to_string(),
            "-t".to_string(),
            format!("{:.6}", end - start),
            "-map".to_string(),
            "1:v:0".to_string(),
        ];

        for (out_idx, &stream_idx) in audio_stream_indices.iter().enumerate() {
            args_mux.extend(["-map".to_string(), format!("0:{}", stream_idx)]);
            if let Some(track) = audio_tracks.get(out_idx) {
                if let Some(name) = &track.name {
                    args_mux.extend([
                        format!("-metadata:s:a:{}", out_idx),
                        format!("title={}", name),
                    ]);
                }
            }
        }

        args_mux.extend([
            "-c:v".to_string(),
            "copy".to_string(),
            "-c:a".to_string(),
            "copy".to_string(),
            "-avoid_negative_ts".to_string(),
            "make_zero".to_string(),
            "-movflags".to_string(),
            "+use_metadata_tags+faststart".to_string(),
            "-map_metadata".to_string(),
            "0".to_string(),
            "-y".to_string(),
            "-progress".to_string(),
            "pipe:2".to_string(),
            output_path.to_string(),
        ]);

        log_debug!(
            phase = "audio_mux",
            "Muxing stream-copied audio over concatenated video"
        );

        let base = current_progress;
        execute_ffmpeg_with_progress(
            ffmpeg_path,
            &args_mux,
            end - start,
            &mut |prog| progress_callback(base + prog / 100.0 * audio_width),
            process_manager,
        )
        .await?;
    }

    progress_callback(100.0);
    let _ = fs::remove_file(&concat_list);

    Ok(())
}

async fn validate_boundaries(
    ffmpeg_path: &Path,
    file: &Path,
    boundaries: &[f64],
    process_manager: &ProcessManager,
) -> bool {
    for &b in boundaries {
        let window_start = (b - 0.2).max(0.0);
        let args = [
            "-hide_banner".to_string(),
            "-v".to_string(),
            "error".to_string(),
            "-xerror".to_string(),
            "-err_detect".to_string(),
            "explode".to_string(),
            "-an".to_string(),
            "-ss".to_string(),
            format!("{:.6}", window_start),
            "-i".to_string(),
            file.to_str().unwrap().to_string(),
            "-t".to_string(),
            "0.6".to_string(),
            "-f".to_string(),
            "null".to_string(),
            "-".to_string(),
        ];
        let child = match new_command(ffmpeg_path)
            .args(&args)
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                log_warn!(error = %e, "Failed to spawn boundary validation");

                return false;
            }
        };
        if let Err(e) = process_manager.attach(&child) {
            log_warn!(error = %e, "Failed to attach boundary validation process");

            return false;
        }
        match child.wait_with_output().await {
            Ok(out) if out.status.success() => {
                log_debug!(boundary = b, "Boundary decode validation passed");
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                if stderr.contains("Error opening output file") || stderr.contains("is not known") {
                    log_warn!(
                        boundary = b,
                        stderr = %stderr.lines().take(3).collect::<Vec<_>>().join(" | "),
                        "Boundary validation could not run; assuming compatible"
                    );
                } else {
                    let snippet = stderr.lines().take(5).collect::<Vec<_>>().join(" | ");

                    log_warn!(
                        boundary = b,
                        stderr = %snippet,
                        "Decode errors at concat boundary - re-encoded parameters incompatible with source"
                    );

                    return false;
                }
            }
            Err(e) => {
                log_warn!(error = %e, "Boundary validation wait failed");

                return false;
            }
        }
    }
    true
}

async fn encode_segment<F>(
    ffmpeg_path: &Path,
    input_path: &str,
    output_path: &Path,
    input_seek: f64,
    output_seek: Option<f64>,
    duration: f64,
    encoder: &str,
    audio_stream_indices: &[usize],
    audio_tracks: &[&AudioTrack],
    force_keyframes: Option<String>,
    with_audio: bool,
    source_hints: Option<&VideoCodecParams>,
    mut progress_callback: F,
    process_manager: &Arc<ProcessManager>,
) -> Result<()>
where
    F: FnMut(f64) + Send + 'static,
{
    log_debug!(
        encoder = %encoder, input_seek, output_seek = ?output_seek, duration,
        "Encoding segment"
    );

    let mut args = vec![
        "-hide_banner".to_string(),
        "-ss".to_string(),
        format!("{:.6}", input_seek),
        "-i".to_string(),
        input_path.to_string(),
    ];

    if let Some(off) = output_seek {
        if off > 0.001 {
            args.extend(["-ss".to_string(), format!("{:.6}", off - 0.0005)]);
        }
    }

    args.extend([
        "-t".to_string(),
        format!("{:.6}", duration),
        "-c:v".to_string(),
        encoder.to_string(),
    ]);

    add_encoder_params(&mut args, encoder, source_hints);

    if let Some(ref keyframes) = force_keyframes {
        args.extend(["-force_key_frames".to_string(), keyframes.clone()]);
    }

    args.extend([
        "-map".to_string(),
        "0:v:0".to_string(),
        "-avoid_negative_ts".to_string(),
        "make_zero".to_string(),
    ]);

    if with_audio {
        args.extend(["-c:a".to_string(), "copy".to_string()]);
        add_audio_mappings_with_metadata(
            &mut args,
            audio_stream_indices,
            audio_tracks,
            false,
            true,
        );
    } else {
        args.push("-an".to_string());
    }
    args.extend([
        "-y".to_string(),
        "-progress".to_string(),
        "pipe:2".to_string(),
        output_path.to_str().unwrap().to_string(),
    ]);

    execute_ffmpeg_with_progress(
        ffmpeg_path,
        &args,
        duration,
        &mut progress_callback,
        process_manager,
    )
    .await
    .map_err(|e| {
        log_error!(encoder = %encoder, error = %e, "Encoder failed");

        let _ = fs::remove_file(output_path);
        AppError::FFmpegError(format!("Encoder failed: {}", e))
    })
}

fn get_matching_encoder(codec: &str) -> String {
    match codec {
        "h264" => "libx264".to_string(),
        "hevc" | "h265" => "libx265".to_string(),
        "av1" => "libsvtav1".to_string(),
        "vp9" => "libvpx-vp9".to_string(),
        "vp8" => "libvpx".to_string(),
        "mpeg4" | "mpeg2video" => "libx264".to_string(),
        _ => "libx264".to_string(),
    }
}

fn add_encoder_params(
    args: &mut Vec<String>,
    encoder: &str,
    source_hints: Option<&VideoCodecParams>,
) {
    match encoder {
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

    let Some(h) = source_hints else { return };
    if matches!(encoder, "libx264" | "libx265") {
        if encoder == "libx264" {
            if let Some(p) = h.profile.as_deref() {
                let mapped = match p.to_lowercase().as_str() {
                    "high" => Some("high"),
                    "main" => Some("main"),
                    "baseline" | "constrained baseline" => Some("baseline"),
                    _ => None,
                };
                if let Some(m) = mapped {
                    args.extend(["-profile:v".to_string(), m.to_string()]);
                }
            }
        }
        if let Some(l) = h.level {
            args.extend(["-level".to_string(), level_to_arg(l)]);
        }
        if let Some(pf) = &h.pix_fmt {
            args.extend(["-pix_fmt".to_string(), pf.replace("yuvj", "yuv")]);
        }
        if let Some(cr) = &h.color_range {
            if cr == "tv" || cr == "pc" {
                args.extend(["-color_range".to_string(), cr.clone()]);
            }
        }
    }
}
