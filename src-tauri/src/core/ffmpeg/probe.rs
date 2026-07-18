use crate::core::ProcessManager;
use crate::core::ffmpeg::mp4_parser::extract_mp4_track_names;
use crate::error::{AppError, Result};
use crate::logger::{log_debug, log_error, log_warn};
use crate::types::metadata::{AudioTrack, VideoMetadata};
use crate::utils::cmd::new_command;
use serde::Deserialize;
use std::path::Path;
use std::process::Stdio;

#[derive(Debug, Deserialize)]
struct FFprobeOutput {
    streams: Vec<FFprobeStream>,
    format: FFprobeFormat,
}

#[derive(Debug, Deserialize)]
struct FFprobeStream {
    #[serde(default)]
    id: Option<String>,
    codec_type: String,
    codec_name: String,
    duration: Option<String>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    r_frame_rate: Option<String>,
    #[serde(default)]
    channels: Option<u32>,
    #[serde(default)]
    tags: Option<FFprobeTags>,
}

#[derive(Debug, Deserialize)]
struct FFprobeTags {
    #[serde(default)]
    title: Option<String>,
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

pub async fn probe_video(
    ffprobe_path: &std::path::Path,
    ffmpeg_path: &std::path::Path,
    video_path: &str,
    process_manager: &ProcessManager,
) -> Result<VideoMetadata> {
    let child = new_command(ffprobe_path)
        .args(&[
            "-loglevel", "error",
            "-hide_banner",
            "-fflags", "nobuffer",
            "-read_intervals", "0%+#10",
            "-print_format", "json",
            "-show_entries", "format=duration,bit_rate:stream=id,index,codec_type,codec_name,duration,r_frame_rate,width,height,channels:stream_tags=title",
            video_path,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
		        log_error!(ffprobe = ?ffprobe_path, error = %e, "Failed to spawn ffprobe");

		        AppError::FFprobeError(format!("Failed to spawn ffprobe at {:?}: {}", ffprobe_path, e))
        })?;

    process_manager.attach(&child)?;

    let output = child.wait_with_output().await.map_err(|e| {
        AppError::FFprobeError(format!(
            "Failed to run ffprobe at {:?}: {}",
            ffprobe_path, e
        ))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        log_error!(ffprobe = ?ffprobe_path, stderr = %stderr, stdout = %stdout, "FFprobe returned non-zero");

        return Err(AppError::FFprobeError(format!(
            "FFprobe failed at {:?}\nstderr: {}\nstdout: {}",
            ffprobe_path, stderr, stdout
        )));
    }

    parse_probe_output(
        &output.stdout,
        video_path,
        ffprobe_path,
        ffmpeg_path,
        process_manager,
    )
    .await
}

async fn parse_probe_output(
    stdout: &[u8],
    video_path: &str,
    ffprobe_path: &Path,
    _ffmpeg_path: &Path,
    process_manager: &ProcessManager,
) -> Result<VideoMetadata> {
    let probe_data: FFprobeOutput = serde_json::from_slice(stdout).map_err(|e| {
        log_error!(error = %e, "Failed to parse ffprobe JSON output");

        AppError::FFprobeError(format!("Failed to parse ffprobe output: {}", e))
    })?;

    let video_stream = probe_data
        .streams
        .iter()
        .find(|s| s.codec_type == "video")
        .ok_or_else(|| {
            log_warn!("No video stream found in probe output");

            AppError::InvalidVideo("No video stream found".to_string())
        })?;

    let mut duration = probe_data
        .format
        .duration
        .as_deref()
        .and_then(|d| d.parse::<f64>().ok())
        .or_else(|| {
            probe_data
                .streams
                .first()?
                .duration
                .as_deref()?
                .parse::<f64>()
                .ok()
        });

    if duration.is_none() || duration == Some(0.0) {
        log_warn!("FFprobe header duration missing; attempting reverse-seek fallback");

        duration =
            get_fallback_duration_reverse_seek(ffprobe_path, video_path, process_manager).await;
    }

    let duration = duration.ok_or_else(|| {
        log_error!("Could not determine video duration after fallback");

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
        .map(|(idx, s)| {
            let title = s.tags.as_ref().and_then(|t| t.title.clone());
            let track_id = parse_stream_id(s.id.as_deref());

            AudioTrack {
                index: idx,
                track_id: track_id.or(idx.try_into().ok()),
                codec: s.codec_name.clone(),
                channels: s.channels.unwrap_or(2),
                name: title,
            }
        })
        .collect();

    let is_mp4 = Path::new(video_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let e = ext.to_lowercase();
            e == "mp4" || e == "m4a" || e == "mov"
        })
        .unwrap_or(false);

    let mut audio_tracks = audio_tracks;
    if is_mp4 {
        let path = video_path.to_string();
        let parsed = tokio::task::spawn_blocking(move || extract_mp4_track_names(&path)).await;
        if let Ok(Ok(track_data)) = parsed {
            for audio_track in audio_tracks.iter_mut() {
                if let Some(ffprobe_id) = audio_track.track_id {
                    if let Some((_, Some(name))) = track_data
                        .iter()
                        .find(|(mp4_id, _)| *mp4_id == Some(ffprobe_id))
                    {
                        audio_track.name = Some(name.clone());
                    }
                }
            }
        }
    }

    Ok(VideoMetadata {
        duration,
        width: video_stream.width.unwrap_or(0),
        height: video_stream.height.unwrap_or(0),
        video_codec: video_stream.codec_name.clone(),
        bitrate,
        fps,
        audio_tracks,
        waveforms: None,
    })
}

async fn get_fallback_duration_reverse_seek(
    ffprobe_path: &Path,
    video_path: &str,
    process_manager: &ProcessManager,
) -> Option<f64> {
    let child = match new_command(ffprobe_path)
        .args(&[
            "-loglevel",
            "error",
            "-hide_banner",
            "-read_intervals",
            "999999%+#1",
            "-select_streams",
            "v:0",
            "-show_entries",
            "packet=pts_time,duration_time",
            "-of",
            "csv=p=0",
            video_path,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            log_debug!(error = %e, "Reverse-seek fallback spawn failed");

            return None;
        }
    };

    let _ = process_manager.attach(&child);

    let output = child.wait_with_output().await.ok()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        log_warn!(stderr = %stderr, "Reverse-seek fallback returned non-zero");

        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;

    let mut max_time = None::<f64>;

    for line in stdout.lines() {
        let mut parts = line.split(',');

        let Some(pts) = parts.next().and_then(|v| v.parse::<f64>().ok()) else {
            continue;
        };

        let duration = parts
            .next()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        let end = pts + duration;

        max_time = Some(max_time.map_or(end, |m| m.max(end)));
    }

    max_time
}

pub async fn get_keyframes(
    ffprobe_path: &Path,
    video_path: &str,
    process_manager: &ProcessManager,
) -> Result<Vec<f64>> {
    let child = new_command(ffprobe_path)
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
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
        		log_error!(ffprobe = ?ffprobe_path, error = %e, "Failed to spawn ffprobe for keyframes");

            AppError::FFprobeError(format!(
                "Failed to spawn ffprobe at {:?}: {}",
                ffprobe_path, e
            ))
        })?;

    process_manager.attach(&child)?;

    let output = child.wait_with_output().await.map_err(|e| {
        AppError::FFprobeError(format!(
            "Failed to run ffprobe at {:?}: {}",
            ffprobe_path, e
        ))
    })?;

    if !output.status.success() {
        let limit_lines = |bytes: &[u8]| -> String {
            const MAX_LINES: u8 = 7;
            let text = String::from_utf8_lossy(bytes);
            let mut result = String::new();
            let mut count = 0;
            for line in text.lines() {
                if count >= MAX_LINES {
                    break;
                }
                if count > 0 {
                    result.push('\n');
                }
                result.push_str(line);
                count += 1;
            }
            let total = text.lines().count();
            if total > MAX_LINES as usize {
                result.push_str(&format!("\n... (truncated, {} total lines)", total));
            }
            result
        };

        let stderr = limit_lines(&output.stderr);
        let stdout = limit_lines(&output.stdout);

        log_error!(ffprobe = ?ffprobe_path, stderr = %stderr, "Keyframe detection returned non-zero");

        return Err(AppError::FFprobeError(format!(
            "FFprobe keyframe detection failed at {:?}\nstderr: {}\nstdout: {}",
            ffprobe_path, stderr, stdout
        )));
    }

    let packets_data: FFprobePacketsOutput =
        serde_json::from_slice(&output.stdout).map_err(|e| {
            log_error!(error = %e, "Failed to parse keyframe JSON");

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

pub fn parse_stream_id(id_str: Option<&str>) -> Option<u32> {
    let id_str = id_str?;
    if let Some(hex_str) = id_str.strip_prefix("0x") {
        u32::from_str_radix(hex_str, 16).ok()
    } else {
        id_str.parse::<u32>().ok()
    }
}

pub fn parse_fps(fps_str: Option<&str>) -> Option<f64> {
    let mut parts = fps_str?.split('/');

    let num = parts.next()?.parse::<f64>().ok()?;
    let den = parts.next()?.parse::<f64>().ok()?;

    if den > 0.0 { Some(num / den) } else { None }
}
