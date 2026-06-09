use crate::error::{AppError, Result};
use crate::ffmpeg::hwaccel;
use crate::logger::log_error;
use regex::Regex;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, fs};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const DEFAULT_X264_QP: u32 = 18;
const DEFAULT_X265_QP: u32 = 18;
const DEFAULT_SVT_AV1_QP: u32 = 20;
const DEFAULT_VP9_CRF: u32 = 18;
const DEFAULT_QSCALE: u32 = 2;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn new_command(program: &Path) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

const DEFAULT_X264_PRESET: &str = "medium";
const DEFAULT_SVT_AV1_PRESET: &str = "6";

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

pub fn execute_ffmpeg_with_progress<F>(
    ffmpeg_path: &Path,
    args: &[String],
    segment_duration: f64,
    mut progress_callback: F,
) -> Result<()>
where
    F: FnMut(f64),
{
    let mut child = new_command(ffmpeg_path)
        .args(args)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::FFmpegError(format!("Failed to spawn ffmpeg: {}", e)))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::FFmpegError("Failed to capture stderr".to_string()))?;

    let reader = BufReader::new(stderr);
    let time_regex = Regex::new(r"time=(\d{2}):(\d{2}):(\d{2}\.\d{2})").unwrap();

    let mut log_buffer: VecDeque<String> = VecDeque::with_capacity(100);
    for line in reader.lines() {
        if let Ok(line) = line {
            if let Some(caps) = time_regex.captures(&line) {
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
    }

    let status = child
        .wait()
        .map_err(|e| AppError::FFmpegError(format!("Failed to wait for ffmpeg: {}", e)))?;

    if !status.success() {
        let trailing_logs = log_buffer.into_iter().collect::<Vec<String>>().join("\n");

        log_error(&format!(
            "FFmpeg exited with status: {}\n--- FFmpeg Stderr Log ---\n{}",
            status,
            if trailing_logs.is_empty() {
                "[No log output captured]"
            } else {
                &trailing_logs
            }
        ));

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
) -> Vec<String> {
    let mut args = Vec::new();

    args.push("-ss".to_string());
    args.push(format!("{:.3}", start));
    args.push("-i".to_string());
    args.push(input_path.to_string());
    args.push("-to".to_string());
    args.push(format!("{:.3}", end - start));

    args.push("-map".to_string());
    args.push("0:v:0".to_string());

    if audio_stream_indices.is_empty() {
        args.push("-an".to_string());
    } else {
        for &idx in audio_stream_indices {
            args.push("-map".to_string());
            args.push(format!("0:{}", idx));
        }
    }

    args.push("-c".to_string());
    args.push("copy".to_string());

    args.push("-y".to_string());

    args.push("-progress".to_string());
    args.push("pipe:2".to_string());

    args.push(output_path.to_string());

    args
}

/// Execute smart cut: 3-part approach (encode boundaries, copy middle, concat, trim)
pub fn execute_smart_cut<F>(
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
    video_codec: &str,
    mut progress_callback: F,
) -> Result<()>
where
    F: FnMut(f64),
{
    // Detect hw capabilities
    let caps = hwaccel::get_hw_capabilities(ffmpeg_path)?;
    let hwaccel_type = hwaccel::select_hwaccel(&caps);
    let encoder_chain = hwaccel::get_encoder_fallback_chain(video_codec, &caps);
    let decoder_chain = hwaccel::get_decoder_fallback_chain(video_codec, &caps);

    let temp_dir = env::temp_dir()
        .join("com.defe.tauri-video-cut")
        .join("temp_segments");
    fs::create_dir_all(&temp_dir)
        .map_err(|e| AppError::ExportError(format!("Failed to create temp dir: {}", e)))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let ext = get_output_extension(input_path);
    let temp_start_encode = temp_dir.join(format!("start_encode_{}.{}", timestamp, ext));
    let temp_middle_copy = temp_dir.join(format!("middle_copy_{}.{}", timestamp, ext));
    let temp_end_encode = temp_dir.join(format!("end_encode_{}.{}", timestamp, ext));
    let temp_concat = temp_dir.join(format!("concat_{}.{}", timestamp, ext));

    // Part 1: Encode K1->K2 if start not on keyframe (40% progress)
    let mut parts = Vec::new();
    let mut current_progress = 0.0;
    let mut working_encoder: Option<String> = None;

    if !start_is_keyframe {
        let duration = k2 - k1;

        let encoder_used = encode_segment_with_fallback(
            ffmpeg_path,
            input_path,
            &temp_start_encode,
            k1,
            duration,
            &encoder_chain,
            &decoder_chain,
            &hwaccel_type,
            audio_stream_indices,
            video_codec,
            Some(format!("expr:gte(t,{:.3})", k2 - k1)),
            |prog| progress_callback(current_progress + prog * 0.4),
        )?;

        working_encoder = Some(encoder_used);
        parts.push(temp_start_encode.to_str().unwrap().to_string());
        current_progress = 40.0;
    }

    // Part 2: Copy K2->K3 (middle, lossless)
    let copy_start = if start_is_keyframe { start } else { k2 };
    let copy_end = if end_is_keyframe { end } else { k3 };
    let copy_duration = copy_end - copy_start;

    let args_copy = vec![
        "-ss".to_string(),
        format!("{:.3}", copy_start),
        "-i".to_string(),
        input_path.to_string(),
        "-t".to_string(),
        format!("{:.3}", copy_duration),
        "-map".to_string(),
        "0:v:0".to_string(),
    ]
    .into_iter()
    .chain(if audio_stream_indices.is_empty() {
        vec!["-an".to_string()]
    } else {
        audio_stream_indices
            .iter()
            .flat_map(|&idx| vec!["-map".to_string(), format!("0:{}", idx)])
            .collect::<Vec<_>>()
    })
    .chain(vec![
        "-c".to_string(),
        "copy".to_string(),
        "-y".to_string(),
        "-progress".to_string(),
        "pipe:2".to_string(),
        temp_middle_copy.to_str().unwrap().to_string(),
    ])
    .collect::<Vec<_>>();

    execute_ffmpeg_with_progress(ffmpeg_path, &args_copy, copy_duration, |prog| {
        progress_callback(current_progress + prog * 0.1);
    })?;

    parts.push(temp_middle_copy.to_str().unwrap().to_string());
    current_progress = 50.0;

    // Part 3: Encode K3->K4 if end not on keyframe
    if !end_is_keyframe {
        let duration = k4 - k3;

        // Use memoized encoder if available, otherwise full chain
        let chain_to_use = if let Some(ref encoder) = working_encoder {
            vec![encoder.clone()]
        } else {
            encoder_chain.clone()
        };

        let encoder_used = encode_segment_with_fallback(
            ffmpeg_path,
            input_path,
            &temp_end_encode,
            k3,
            duration,
            &chain_to_use,
            &decoder_chain,
            &hwaccel_type,
            audio_stream_indices,
            video_codec,
            Some("expr:eq(n,0)".to_string()),
            |prog| progress_callback(current_progress + prog * 0.4),
        )?;

        // Update memoized encoder if this was first encode
        if working_encoder.is_none() {
            working_encoder = Some(encoder_used);
        }

        parts.push(temp_end_encode.to_str().unwrap().to_string());
    }

    // Part 4: Concat all parts (5% progress)
    let concat_content = parts
        .iter()
        .map(|p| format!("file '{}'", p.replace('\\', "/")))
        .collect::<Vec<_>>()
        .join("\n");

    let concat_list = temp_dir.join(format!("concat_list_{}.txt", timestamp));
    fs::write(&concat_list, &concat_content)
        .map_err(|e| AppError::ExportError(format!("Failed to write concat list: {}", e)))?;

    let args_concat = vec![
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
        "-map".to_string(),
        "0:a?".to_string(),
        "-c".to_string(),
        "copy".to_string(),
        "-y".to_string(),
        temp_concat.to_str().unwrap().to_string(),
    ];

    let mut child = new_command(ffmpeg_path)
        .args(&args_concat)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::FFmpegError(format!("Failed to spawn ffmpeg concat: {}", e)))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::FFmpegError("Failed to capture concat stderr".to_string()))?;

    let reader = BufReader::new(stderr);
    let mut concat_log = Vec::new();
    for line in reader.lines() {
        if let Ok(line) = line {
            concat_log.push(line);
        }
    }

    let status = child
        .wait()
        .map_err(|e| AppError::FFmpegError(format!("Failed to wait for concat: {}", e)))?;

    if !status.success() {
        let log_output = concat_log.join("\n");

        log_error(&format!(
            "Concat failed with status: {}\n=== FFmpeg stderr ===\n{}",
            status, log_output
        ));

        return Err(AppError::FFmpegError(format!(
            "Concat failed with status: {}\n=== FFmpeg stderr ===\n{}",
            status, log_output
        )));
    }

    progress_callback(95.0);

    // Part 5: Trim to exact start/end (5% progress)

    let trim_start = if start_is_keyframe { 0.0 } else { start - k1 };
    let trim_duration = end - start;

    let args_trim = vec![
        "-ss".to_string(),
        format!("{:.3}", trim_start),
        "-i".to_string(),
        temp_concat.to_str().unwrap().to_string(),
        "-t".to_string(),
        format!("{:.3}", trim_duration),
        "-map".to_string(),
        "0".to_string(),
        "-c".to_string(),
        "copy".to_string(),
        "-y".to_string(),
        output_path.to_string(),
    ];

    let status = new_command(ffmpeg_path)
        .args(&args_trim)
        .stderr(Stdio::null())
        .status()
        .map_err(|e| AppError::FFmpegError(format!("Failed to spawn ffmpeg trim: {}", e)))?;

    if !status.success() {
        log_error(&format!("Trim failed with status: {}", status));

        return Err(AppError::FFmpegError(format!(
            "Trim failed with status: {}",
            status
        )));
    }

    progress_callback(100.0);

    let _ = fs::remove_file(&temp_start_encode);
    let _ = fs::remove_file(&temp_middle_copy);
    let _ = fs::remove_file(&temp_end_encode);
    let _ = fs::remove_file(&temp_concat);
    let _ = fs::remove_file(&concat_list);

    Ok(())
}

fn encode_segment_with_fallback<F>(
    ffmpeg_path: &Path,
    input_path: &str,
    output_path: &Path,
    start_time: f64,
    duration: f64,
    encoder_chain: &[String],
    decoder_chain: &[Option<String>],
    hwaccel_type: &hwaccel::HwAccelType,
    audio_stream_indices: &[usize],
    video_codec: &str,
    force_keyframes: Option<String>,
    mut progress_callback: F,
) -> Result<String>
where
    F: FnMut(f64),
{
    let mut last_error = None;

    // Try each encoder
    for (enc_idx, encoder) in encoder_chain.iter().enumerate() {
        // Try each decoder for this encoder
        for (dec_idx, decoder) in decoder_chain.iter().enumerate() {
            let mut args = vec![];

            // Add hwaccel + decoder only if hw decoder specified
            let is_hw_decoder = decoder.as_ref().map_or(false, |d| d.contains("cuvid"));

            if is_hw_decoder {
                args.extend(hwaccel::build_hwaccel_args(hwaccel_type));

                // Add explicit hw decoder
                if let Some(dec) = decoder {
                    args.push("-c:v".to_string());
                    args.push(dec.clone());
                }
            } else if let Some(dec) = decoder {
                // Software decoder like libdav1d - no hwaccel needed
                args.push("-c:v".to_string());
                args.push(dec.clone());
            }

            args.push("-ss".to_string());
            args.push(format!("{:.3}", start_time));
            args.push("-i".to_string());
            args.push(input_path.to_string());
            args.push("-t".to_string());
            args.push(format!("{:.3}", duration));
            args.push("-c:v".to_string());
            args.push(encoder.clone());

            add_encoder_params(&mut args, encoder, video_codec);

            if let Some(ref keyframes) = force_keyframes {
                args.push("-force_key_frames".to_string());
                args.push(keyframes.clone());
            }

            args.push("-c:a".to_string());
            args.push("copy".to_string());
            args.push("-map".to_string());
            args.push("0:v:0".to_string());

            if audio_stream_indices.is_empty() {
                args.push("-an".to_string());
            } else {
                for &stream_idx in audio_stream_indices {
                    args.push("-map".to_string());
                    args.push(format!("0:{}", stream_idx));
                }
            }

            args.push("-y".to_string());
            args.push("-progress".to_string());
            args.push("pipe:2".to_string());
            args.push(output_path.to_str().unwrap().to_string());

            match execute_ffmpeg_with_progress(ffmpeg_path, &args, duration, &mut progress_callback)
            {
                Ok(_) => {
                    return Ok(encoder.clone());
                }
                Err(e) => {
                    last_error = Some(e);

                    // Clean up failed output
                    let _ = fs::remove_file(output_path);

                    if dec_idx + 1 >= decoder_chain.len() && enc_idx + 1 >= encoder_chain.len() {
                        break;
                    }

                    if dec_idx + 1 >= decoder_chain.len() {
                        break; // Move to next encoder
                    }
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        AppError::FFmpegError(
            "All encoder/decoder combinations in fallback chain failed".to_string(),
        )
    }))
}

fn add_encoder_params(args: &mut Vec<String>, encoder: &str, _video_codec: &str) {
    match encoder {
        "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" => {
            args.push("-preset".to_string());
            args.push("p4".to_string()); // Medium quality/speed
            args.push("-cq".to_string());
            args.push(DEFAULT_X264_QP.to_string());
        }
        "libx264" | "libx265" => {
            args.push("-preset".to_string());
            args.push(DEFAULT_X264_PRESET.to_string());
            args.push("-qp".to_string());
            args.push(if encoder == "libx264" {
                DEFAULT_X264_QP.to_string()
            } else {
                DEFAULT_X265_QP.to_string()
            });
        }
        "libsvtav1" => {
            args.push("-preset".to_string());
            args.push(DEFAULT_SVT_AV1_PRESET.to_string());
            args.push("-qp".to_string());
            args.push(DEFAULT_SVT_AV1_QP.to_string());
        }
        "libvpx-vp9" => {
            args.push("-crf".to_string());
            args.push(DEFAULT_VP9_CRF.to_string());
            args.push("-b:v".to_string());
            args.push("0".to_string());
        }
        _ => {
            args.push("-qscale:v".to_string());
            args.push(DEFAULT_QSCALE.to_string());
        }
    }
}

pub fn get_output_extension(input_path: &str) -> String {
    Path::new(input_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp4")
        .to_string()
}
