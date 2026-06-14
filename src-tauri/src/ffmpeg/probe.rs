use crate::error::{AppError, Result};
use crate::logger::{log_error, log_info, log_warn};
use crate::models::{AudioTrack, VideoMetadata};
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
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
    #[serde(default)]
    id: Option<String>,
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

pub fn probe_video(ffprobe_path: &std::path::Path, video_path: &str) -> Result<VideoMetadata> {
    let output = new_command(ffprobe_path)
        .args(&[
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_entries",
            "format=duration,bit_rate:stream=id,index,codec_type,codec_name,r_frame_rate,width,height,channels:stream_tags=title",
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

    // Try to extract mp4 track names if it's an mp4 file
    let mut audio_tracks = audio_tracks;
    let is_mp4 = Path::new(video_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let e = ext.to_lowercase();
            e == "mp4" || e == "m4a" || e == "mov" // Apple devices often hide this metadata in .mov too!
        })
        .unwrap_or(false);

    if is_mp4 {
        if let Ok(track_data) = extract_mp4_track_names(video_path) {
            for audio_track in audio_tracks.iter_mut() {
                if let Some(ffprobe_id) = audio_track.track_id {
                    let matched = track_data
                        .iter()
                        .find(|(mp4_id, _)| *mp4_id == Some(ffprobe_id));

                    if let Some((_, Some(name))) = matched {
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

fn parse_stream_id(id_str: Option<&str>) -> Option<u32> {
    let id_str = id_str?;
    if let Some(hex_str) = id_str.strip_prefix("0x") {
        u32::from_str_radix(hex_str, 16).ok()
    } else {
        id_str.parse::<u32>().ok()
    }
}

fn parse_fps(fps_str: Option<&str>) -> Option<f64> {
    let mut parts = fps_str?.split('/');

    let num = parts.next()?.parse::<f64>().ok()?;
    let den = parts.next()?.parse::<f64>().ok()?;

    if den > 0.0 { Some(num / den) } else { None }
}

/// Parse MP4 track names from moov->trak->udta->name atoms \
/// See https://trac.ffmpeg.org/ticket/9438 \
/// Skips heavy mdat (media data) to avoid loading into memory \
/// Returns Vec of (track_id, name) pairs
fn extract_mp4_track_names(video_path: &str) -> Result<Vec<(Option<u32>, Option<String>)>> {
    let mut file = File::open(video_path)
        .map_err(|e| AppError::FFprobeError(format!("Failed to open mp4 file: {}", e)))?;

    let mut track_data = Vec::new();

    // Read file in chunks, looking for atoms
    let file_size = file
        .metadata()
        .map_err(|e| AppError::FFprobeError(format!("Failed to get file metadata: {}", e)))?
        .len();

    let mut pos = 0u64;

    while pos < file_size {
        file.seek(SeekFrom::Start(pos))
            .map_err(|e| AppError::FFprobeError(format!("Failed to seek in mp4: {}", e)))?;

        let header = read_atom_header(&mut file, pos, file_size)?;

        if header.size == 0 {
            return Err(AppError::FFprobeError(
                "Atom size resolved to 0, infinite loop prevented".to_string(),
            ));
        }

        // Found moov atom - parse it for trak atoms
        if &header.atom_type == b"moov" {
            let moov_end = pos + header.size;
            let mut moov_pos = pos + header.header_size;

            while moov_pos < moov_end {
                file.seek(SeekFrom::Start(moov_pos)).map_err(|e| {
                    AppError::FFprobeError(format!("Failed to seek to trak at {}: {}", moov_pos, e))
                })?;

                let trak_header = read_atom_header(&mut file, moov_pos, moov_end)?;

                if trak_header.size == 0 {
                    return Err(AppError::FFprobeError(
                        "Trak size 0, infinite loop prevented".to_string(),
                    ));
                }

                // Found trak atom - extract track_id and name
                if &trak_header.atom_type == b"trak" {
                    let track_id = extract_track_id(&mut file, moov_pos, &trak_header)?;
                    let track_name = parse_trak_name(&mut file, moov_pos, &trak_header)?;

                    track_data.push((track_id, track_name));
                }

                moov_pos += trak_header.size;
            }

            break; // Done after processing moov
        }

        // Skip mdat and other large atoms without reading into memory
        pos += header.size;
    }

    for (idx, (track_id, name)) in track_data.iter().enumerate() {
        match (track_id, name) {
            (Some(id), Some(n)) => {
                log_info(&format!("[MP4 Parser] Track {} ID={}: '{}'", idx, id, n))
            }
            (Some(id), None) => {
                log_info(&format!("[MP4 Parser] Track {} ID={}: <no name>", idx, id))
            }
            (None, Some(n)) => {
                log_info(&format!("[MP4 Parser] Track {} ID=<unknown>: '{}'", idx, n))
            }
            (None, None) => log_info(&format!(
                "[MP4 Parser] Track {} ID=<unknown>: <no name>",
                idx
            )),
        }
    }

    Ok(track_data)
}

/// Extract track ID from tkhd atom in trak
fn extract_track_id(
    file: &mut File,
    trak_start: u64,
    trak_header: &AtomHeader,
) -> Result<Option<u32>> {
    let trak_end = trak_start + trak_header.size;
    let mut pos = trak_start + trak_header.header_size;

    while pos < trak_end {
        file.seek(SeekFrom::Start(pos)).map_err(|e| {
            AppError::FFprobeError(format!(
                "Failed to seek in trak (looking for tkhd) at {}: {}",
                pos, e
            ))
        })?;

        let header = read_atom_header(file, pos, trak_end)?;

        if header.size == 0 || header.size > trak_header.size {
            break;
        }

        if &header.atom_type == b"tkhd" {
            // tkhd layout: version(1) + flags(3) + created(4/8) + modified(4/8) + track_id(4)
            let mut version_flags = [0u8; 4];
            if file.read_exact(&mut version_flags).is_ok() {
                let version = version_flags[0];
                let time_size = if version == 1 { 8u64 } else { 4u64 };

                // Skip creation_time and modification_time
                file.seek(SeekFrom::Current((time_size * 2) as i64))
                    .map_err(|e| {
                        AppError::FFprobeError(format!("Failed to skip tkhd timestamps: {}", e))
                    })?;

                // Read track_id (4 bytes)
                let mut track_id_buf = [0u8; 4];
                if file.read_exact(&mut track_id_buf).is_ok() {
                    let track_id = u32::from_be_bytes(track_id_buf);
                    return Ok(Some(track_id));
                }
            }
            break;
        }

        pos += header.size;
    }

    Ok(None)
}

/// Atom header info with size and whether it's 64-bit
struct AtomHeader {
    size: u64,
    atom_type: [u8; 4],
    header_size: u64,
}

/// Read atom header (size + type), handling 64-bit large size
fn read_atom_header(file: &mut File, pos: u64, container_end: u64) -> Result<AtomHeader> {
    let mut size_buf = [0u8; 4];
    file.read_exact(&mut size_buf)
        .map_err(|e| AppError::FFprobeError(format!("Failed to read atom size: {}", e)))?;

    let size32 = u32::from_be_bytes(size_buf) as u64;

    // Read type (always at bytes 4-7)
    let mut atom_type = [0u8; 4];
    file.read_exact(&mut atom_type)
        .map_err(|e| AppError::FFprobeError(format!("Failed to read atom type: {}", e)))?;

    let (size, header_size) = if size32 == 1 {
        // 64-bit large size follows type
        let mut size64_buf = [0u8; 8];
        file.read_exact(&mut size64_buf).map_err(|e| {
            AppError::FFprobeError(format!("Failed to read 64-bit atom size: {}", e))
        })?;
        let size64 = u64::from_be_bytes(size64_buf);

        if size64 == 0 {
            return Err(AppError::FFprobeError(
                "64-bit atom size cannot be 0".to_string(),
            ));
        }

        (size64, 16u64) // 4 (size32=1) + 4 (type) + 8 (size64)
    } else if size32 == 0 {
        // Extends to end of container/file
        let size = container_end - pos;

        if size < 8 {
            return Err(AppError::FFprobeError(format!(
                "Invalid zero-size atom: {}",
                size
            )));
        }

        (size, 8u64)
    } else if size32 < 8 {
        return Err(AppError::FFprobeError(format!(
            "Invalid atom size: {}",
            size32
        )));
    } else {
        (size32, 8u64) // 4 (size) + 4 (type)
    };

    Ok(AtomHeader {
        size,
        atom_type,
        header_size,
    })
}

fn parse_trak_name(
    file: &mut File,
    trak_start: u64,
    trak_header: &AtomHeader,
) -> Result<Option<String>> {
    let trak_end = trak_start + trak_header.size;
    let mut pos = trak_start + trak_header.header_size;

    while pos < trak_end {
        file.seek(SeekFrom::Start(pos)).map_err(|e| {
            AppError::FFprobeError(format!("Failed to seek in trak at {}: {}", pos, e))
        })?;

        let header = read_atom_header(file, pos, trak_end)?;

        if header.size == 0 || header.size > trak_header.size {
            break;
        }

        // Found udta atom - try multiple paths
        if &header.atom_type == b"udta" {
            // Try path 1: udta -> name (plain text)
            if let Some(name) = try_udta_name(file, pos, &header)? {
                return Ok(Some(name));
            }

            // Try path 2: udta -> meta -> ilst -> ©nam -> data
            if let Some(name) = try_udta_meta_ilst_nam(file, pos, &header)? {
                return Ok(Some(name));
            }

            // Try path 3: udta -> title
            if let Some(name) = try_udta_title(file, pos, &header)? {
                return Ok(Some(name));
            }
        }

        // Found mdia atom - check for nested udta
        if &header.atom_type == b"mdia" {
            if let Some(name) = parse_mdia_udta(file, pos, &header)? {
                return Ok(Some(name));
            }
        }

        pos += header.size;
    }

    Ok(None)
}

/// Parse trak -> mdia -> udta path (professional editing suites)
fn parse_mdia_udta(
    file: &mut File,
    mdia_start: u64,
    mdia_header: &AtomHeader,
) -> Result<Option<String>> {
    let mdia_end = mdia_start + mdia_header.size;
    let mut pos = mdia_start + mdia_header.header_size;

    while pos < mdia_end {
        file.seek(SeekFrom::Start(pos)).map_err(|e| {
            AppError::FFprobeError(format!("Failed to seek in mdia at {}: {}", pos, e))
        })?;

        let header = read_atom_header(file, pos, mdia_end)?;

        if header.size == 0 || header.size > mdia_header.size {
            break;
        }

        if &header.atom_type == b"udta" {
            // Try all udta paths
            if let Some(name) = try_udta_name(file, pos, &header)? {
                return Ok(Some(name));
            }

            if let Some(name) = try_udta_meta_ilst_nam(file, pos, &header)? {
                return Ok(Some(name));
            }

            if let Some(name) = try_udta_title(file, pos, &header)? {
                return Ok(Some(name));
            }
        }

        pos += header.size;
    }

    Ok(None)
}

/// Path 1: trak -> udta -> name (plain text after 8 byte header)
fn try_udta_name(
    file: &mut File,
    udta_start: u64,
    udta_header: &AtomHeader,
) -> Result<Option<String>> {
    let udta_end = udta_start + udta_header.size;
    let mut udta_pos = udta_start + udta_header.header_size;

    while udta_pos < udta_end {
        file.seek(SeekFrom::Start(udta_pos)).map_err(|e| {
            AppError::FFprobeError(format!(
                "Failed to seek in udta (name path) at {}: {}",
                udta_pos, e
            ))
        })?;

        let header = read_atom_header(file, udta_pos, udta_end)?;

        if header.size == 0 || header.size > udta_header.size {
            break;
        }

        if &header.atom_type == b"name" {
            // Calculate data size with underflow check
            if header.size < header.header_size {
                break;
            }

            let data_size = (header.size - header.header_size) as usize;
            if data_size > 0 && data_size < 1024 {
                let mut name_data = vec![0u8; data_size];
                if file.read_exact(&mut name_data).is_ok() {
                    let name_str = String::from_utf8_lossy(&name_data)
                        .trim_end_matches('\0')
                        .trim()
                        .to_string();
                    log_info(&format!(
                        "[MP4 Parser] Extracted (udta->name): '{}'",
                        name_str
                    ));
                    if !name_str.is_empty() {
                        return Ok(Some(name_str));
                    }
                }
            }
        }

        udta_pos += header.size;
    }

    Ok(None)
}

/// Path 2: trak -> udta -> meta -> ilst -> ©nam -> data
fn try_udta_meta_ilst_nam(
    file: &mut File,
    udta_start: u64,
    udta_header: &AtomHeader,
) -> Result<Option<String>> {
    let udta_end = udta_start + udta_header.size;
    let mut udta_pos = udta_start + udta_header.header_size;

    while udta_pos < udta_end {
        file.seek(SeekFrom::Start(udta_pos)).map_err(|e| {
            AppError::FFprobeError(format!(
                "Failed to seek in udta (meta path) at {}: {}",
                udta_pos, e
            ))
        })?;

        let meta_header = read_atom_header(file, udta_pos, udta_end)?;

        if meta_header.size == 0 || meta_header.size > udta_header.size {
            break;
        }

        if &meta_header.atom_type == b"meta" {
            // meta has version/flags (4 bytes) after header
            let meta_end = udta_pos + meta_header.size;
            let mut meta_pos = udta_pos + meta_header.header_size + 4; // Skip header + version/flags

            while meta_pos < meta_end {
                file.seek(SeekFrom::Start(meta_pos)).map_err(|e| {
                    AppError::FFprobeError(format!("Failed to seek in meta at {}: {}", meta_pos, e))
                })?;

                let ilst_header = read_atom_header(file, meta_pos, meta_end)?;

                if ilst_header.size == 0 || ilst_header.size > meta_header.size {
                    break;
                }

                if &ilst_header.atom_type == b"ilst" {
                    let ilst_end = meta_pos + ilst_header.size;
                    let mut ilst_pos = meta_pos + ilst_header.header_size;

                    while ilst_pos < ilst_end {
                        file.seek(SeekFrom::Start(ilst_pos)).map_err(|e| {
                            AppError::FFprobeError(format!(
                                "Failed to seek in ilst at {}: {}",
                                ilst_pos, e
                            ))
                        })?;

                        let nam_header = read_atom_header(file, ilst_pos, ilst_end)?;

                        if nam_header.size == 0 || nam_header.size > ilst_header.size {
                            break;
                        }

                        if &nam_header.atom_type == b"\xa9nam" {
                            let nam_end = ilst_pos + nam_header.size;
                            let mut nam_pos = ilst_pos + nam_header.header_size;

                            while nam_pos < nam_end {
                                file.seek(SeekFrom::Start(nam_pos)).map_err(|e| {
                                    AppError::FFprobeError(format!(
                                        "Failed to seek in ©nam at {}: {}",
                                        nam_pos, e
                                    ))
                                })?;

                                let data_header = read_atom_header(file, nam_pos, nam_end)?;

                                if data_header.size == 0 || data_header.size > nam_header.size {
                                    break;
                                }

                                if &data_header.atom_type == b"data" {
                                    // data has type flags (4 bytes) after header - check underflow
                                    let header_and_flags = data_header.header_size + 4;
                                    if data_header.size < header_and_flags {
                                        break;
                                    }

                                    let text_size = (data_header.size - header_and_flags) as usize;
                                    if text_size > 0 && text_size < 1024 {
                                        let mut type_flags = [0u8; 4];
                                        if file.read_exact(&mut type_flags).is_ok() {
                                            let mut name_data = vec![0u8; text_size];
                                            if file.read_exact(&mut name_data).is_ok() {
                                                let name_str = String::from_utf8_lossy(&name_data)
                                                    .trim_end_matches('\0')
                                                    .trim()
                                                    .to_string();
                                                log_info(&format!(
                                                    "[MP4 Parser] Extracted (udta->meta->ilst->©nam->data): '{}'",
                                                    name_str
                                                ));
                                                if !name_str.is_empty() {
                                                    return Ok(Some(name_str));
                                                }
                                            }
                                        }
                                    }
                                }

                                nam_pos += data_header.size;
                            }
                        }

                        ilst_pos += nam_header.size;
                    }
                }

                meta_pos += ilst_header.size;
            }
        }

        udta_pos += meta_header.size;
    }

    Ok(None)
}

/// Path 3: trak -> udta -> title (version/flags + language pad + text)
fn try_udta_title(
    file: &mut File,
    udta_start: u64,
    udta_header: &AtomHeader,
) -> Result<Option<String>> {
    let udta_end = udta_start + udta_header.size;
    let mut udta_pos = udta_start + udta_header.header_size;

    while udta_pos < udta_end {
        file.seek(SeekFrom::Start(udta_pos)).map_err(|e| {
            AppError::FFprobeError(format!(
                "Failed to seek in udta (title path) at {}: {}",
                udta_pos, e
            ))
        })?;

        let header = read_atom_header(file, udta_pos, udta_end)?;

        if header.size == 0 || header.size > udta_header.size {
            break;
        }

        if &header.atom_type == b"titl" {
            // title has version/flags (4 bytes) + language pad (2 bytes) - check underflow
            let skip_size = header.header_size + 4 + 2;
            if header.size < skip_size {
                break;
            }

            let text_size = (header.size - skip_size) as usize;
            if text_size > 0 && text_size < 1024 {
                let mut skip_bytes = [0u8; 6]; // version/flags + lang pad
                if file.read_exact(&mut skip_bytes).is_ok() {
                    let mut name_data = vec![0u8; text_size];
                    if file.read_exact(&mut name_data).is_ok() {
                        let name_str = String::from_utf8_lossy(&name_data)
                            .trim_end_matches('\0')
                            .trim()
                            .to_string();

                        log_info(&format!(
                            "[MP4 Parser] Extracted (udta->title): '{}'",
                            name_str
                        ));

                        if !name_str.is_empty() {
                            return Ok(Some(name_str));
                        }
                    }
                }
            }
        }

        udta_pos += header.size;
    }

    Ok(None)
}
