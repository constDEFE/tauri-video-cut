use crate::error::{AppError, Result};
use crate::logger::log_debug;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// Parse MP4 track names from moov->trak->udta->name atoms
pub fn extract_mp4_track_names(video_path: &str) -> Result<Vec<(Option<u32>, Option<String>)>> {
    log_debug!(video = %video_path, "Parsing MP4 track names");

    let mut file = File::open(video_path)
        .map_err(|e| AppError::FFprobeError(format!("Failed to open mp4 file: {}", e)))?;

    let mut track_data = Vec::new();

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

                if &trak_header.atom_type == b"trak" {
                    let track_id = extract_track_id(&mut file, moov_pos, &trak_header)?;
                    let track_name = parse_trak_name(&mut file, moov_pos, &trak_header)?;

                    track_data.push((track_id, track_name));
                }

                moov_pos += trak_header.size;
            }

            break;
        }

        pos += header.size;
    }

    log_debug!(track_count = track_data.len(), "MP4 track parsing complete");

    Ok(track_data)
}

struct AtomHeader {
    size: u64,
    atom_type: [u8; 4],
    header_size: u64,
}

fn read_atom_header(file: &mut File, pos: u64, container_end: u64) -> Result<AtomHeader> {
    let mut size_buf = [0u8; 4];
    file.read_exact(&mut size_buf)
        .map_err(|e| AppError::FFprobeError(format!("Failed to read atom size: {}", e)))?;

    let size32 = u32::from_be_bytes(size_buf) as u64;

    let mut atom_type = [0u8; 4];
    file.read_exact(&mut atom_type)
        .map_err(|e| AppError::FFprobeError(format!("Failed to read atom type: {}", e)))?;

    let (size, header_size) = if size32 == 1 {
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

        (size64, 16u64)
    } else if size32 == 0 {
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
        (size32, 8u64)
    };

    Ok(AtomHeader {
        size,
        atom_type,
        header_size,
    })
}

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
            let mut version_flags = [0u8; 4];
            if file.read_exact(&mut version_flags).is_ok() {
                let version = version_flags[0];
                let time_size = if version == 1 { 8u64 } else { 4u64 };

                file.seek(SeekFrom::Current((time_size * 2) as i64))
                    .map_err(|e| {
                        AppError::FFprobeError(format!("Failed to skip tkhd timestamps: {}", e))
                    })?;

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

        if &header.atom_type == b"udta" {
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

        if &header.atom_type == b"mdia" {
            if let Some(name) = parse_mdia_udta(file, pos, &header)? {
                return Ok(Some(name));
            }
        }

        pos += header.size;
    }

    Ok(None)
}

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

                    log_debug!(atom_path = "udta->name", name = %name_str, "Extracted track name");

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
            let meta_end = udta_pos + meta_header.size;
            let mut meta_pos = udta_pos + meta_header.header_size + 4;

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

                                                log_debug!(atom_path = "udta->meta->ilst->©nam->data", name = %name_str, "Extracted track name");

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
            let skip_size = header.header_size + 4 + 2;
            if header.size < skip_size {
                break;
            }

            let text_size = (header.size - skip_size) as usize;
            if text_size > 0 && text_size < 1024 {
                let mut skip_bytes = [0u8; 6];
                if file.read_exact(&mut skip_bytes).is_ok() {
                    let mut name_data = vec![0u8; text_size];
                    if file.read_exact(&mut name_data).is_ok() {
                        let name_str = String::from_utf8_lossy(&name_data)
                            .trim_end_matches('\0')
                            .trim()
                            .to_string();

                        log_debug!(atom_path = "udta->titl", name = %name_str, "Extracted track name");

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
