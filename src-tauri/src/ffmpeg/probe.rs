use crate::error::{AppError, Result};
use crate::logger::{log_error, log_warn};
use crate::models::{AudioTrack, VideoMetadata};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn new_command(program: &Path) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[derive(Debug, Deserialize)]
struct FFprobeOutput {
    streams: Vec<FFprobeStream>,
    format: FFprobeFormat,
}

#[derive(Debug, Deserialize)]
struct FFprobeStream {
    codec_type: String,
    codec_name: String,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    r_frame_rate: Option<String>,
    #[serde(default)]
    channels: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct FFprobeFormat {
    duration: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FFprobePacketsOutput {
    packets: Vec<FFprobePacket>,
}

#[derive(Debug, Deserialize)]
struct FFprobePacket {
    pts_time: Option<String>,
    flags: Option<String>,
}

pub fn probe_video(ffprobe_path: &std::path::Path, video_path: &str) -> Result<VideoMetadata> {
    let output = new_command(ffprobe_path)
        .args(&[
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_entries",
            "format=duration,bit_rate:stream=index,codec_type,codec_name,r_frame_rate,width,height,channels",
            video_path,
        ])
        .output()
        .map_err(|e| {
            AppError::FFprobeError(format!(
                "Failed to run ffprobe at {:?}: {}",
                ffprobe_path, e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        log_error(&format!(
            "FFprobe failed at {:?}\nstderr: {}\nstdout: {}",
            ffprobe_path, stderr, stdout
        ));

        return Err(AppError::FFprobeError(format!(
            "FFprobe failed at {:?}\nstderr: {}\nstdout: {}",
            ffprobe_path, stderr, stdout
        )));
    }

    let probe_data: FFprobeOutput = serde_json::from_slice(&output.stdout).map_err(|e| {
        log_error(&format!("Failed to parse ffprobe output: {}", e));
        AppError::FFprobeError(format!("Failed to parse ffprobe output: {}", e))
    })?;

    let video_stream = probe_data
        .streams
        .iter()
        .find(|s| s.codec_type == "video")
        .ok_or_else(|| {
            log_warn("No video stream found");
            AppError::InvalidVideo("No video stream found".to_string())
        })?;

    let duration = probe_data
        .format
        .duration
        .as_ref()
        .and_then(|d| d.parse::<f64>().ok())
        .ok_or_else(|| {
            log_warn("Could not determine video duration");
            AppError::InvalidVideo("Could not determine video duration".to_string())
        })?;

    let bitrate = probe_data
        .format
        .bit_rate
        .as_ref()
        .and_then(|b| b.parse::<u64>().ok())
        .unwrap_or(0);

    let fps = parse_fps(video_stream.r_frame_rate.as_deref()).unwrap_or(30.0);

    let audio_tracks: Vec<AudioTrack> = probe_data
        .streams
        .iter()
        .enumerate()
        .filter(|(_, s)| s.codec_type == "audio")
        .map(|(idx, s)| AudioTrack {
            index: idx,
            codec: s.codec_name.clone(),
            channels: s.channels.unwrap_or(2),
        })
        .collect();

    Ok(VideoMetadata {
        duration,
        width: video_stream.width.unwrap_or(0),
        height: video_stream.height.unwrap_or(0),
        video_codec: video_stream.codec_name.clone(),
        bitrate,
        fps,
        audio_tracks,
    })
}

pub fn get_keyframes(ffprobe_path: &std::path::Path, video_path: &str) -> Result<Vec<f64>> {
    let output = new_command(ffprobe_path)
        .args(&[
            "-v",
            "quiet",
            "-select_streams",
            "v:0",
            "-show_entries",
            "packet=pts_time,flags",
            "-of",
            "json",
            video_path,
        ])
        .output()
        .map_err(|e| {
            AppError::FFprobeError(format!(
                "Failed to run ffprobe at {:?}: {}",
                ffprobe_path, e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        log_error(&format!(
            "FFprobe keyframe detection failed at {:?}\nstderr: {}\nstdout: {}",
            ffprobe_path, stderr, stdout
        ));

        return Err(AppError::FFprobeError(format!(
            "FFprobe keyframe detection failed at {:?}\nstderr: {}\nstdout: {}",
            ffprobe_path, stderr, stdout
        )));
    }

    let packets_data: FFprobePacketsOutput =
        serde_json::from_slice(&output.stdout).map_err(|e| {
            log_error(&format!("Failed to parse keyframe data: {}", e));
            AppError::FFprobeError(format!("Failed to parse keyframe data: {}", e))
        })?;

    let mut keyframes = Vec::new();
    for packet in packets_data.packets {
        if let Some(flags) = packet.flags {
            if flags.contains('K') {
                if let Some(time_str) = packet.pts_time {
                    if let Ok(time) = time_str.parse::<f64>() {
                        keyframes.push(time);
                    }
                }
            }
        }
    }

    Ok(keyframes)
}

fn parse_fps(fps_str: Option<&str>) -> Option<f64> {
    let fps_str = fps_str?;
    let parts: Vec<&str> = fps_str.split('/').collect();
    if parts.len() == 2 {
        let num = parts[0].parse::<f64>().ok()?;
        let den = parts[1].parse::<f64>().ok()?;
        if den > 0.0 {
            return Some(num / den);
        }
    }
    None
}
