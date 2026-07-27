use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::core::waveform::model::TOTAL_WAVEFORM_POINTS;
use crate::utils::paths::app_temp_dir;

pub const CACHE_VERSION: u32 = 1;
pub const CACHE_HEADER_SIZE: u64 = 8; // magic(4) + version(4)
pub const POINT_SIZE: u64 = 6; // rms_l, rms_r, up_l, down_l, up_r, down_r

pub fn get_cache_path(
    video_path: &str,
    track_index: u32,
    target_rate: u32,
    duration: f64,
) -> Result<PathBuf, String> {
    let dir = app_temp_dir().join("cache").join("waveforms");
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create cache dir: {e}"))?;

    let mut state: u64 = 0xcbf29ce484222325;

    fn mix(s: &mut u64, bytes: &[u8]) {
        for &b in bytes {
            *s ^= b as u64;
            *s = s.wrapping_mul(0x100000001b3);
        }
        *s ^= b'|' as u64;
        *s = s.wrapping_mul(0x100000001b3);
    }

    let duration_ms = (duration * 1000.0).round() as u64;

    mix(&mut state, video_path.as_bytes());
    mix(&mut state, &track_index.to_le_bytes());
    mix(&mut state, &target_rate.to_le_bytes());
    mix(&mut state, &(TOTAL_WAVEFORM_POINTS as u32).to_le_bytes());
    mix(&mut state, &duration_ms.to_le_bytes());

    if let Ok(meta) = std::fs::metadata(video_path) {
        mix(&mut state, &meta.len().to_le_bytes());
        if let Ok(mtime) = meta.modified() {
            if let Ok(dur) = mtime.duration_since(std::time::UNIX_EPOCH) {
                mix(&mut state, &dur.as_secs().to_le_bytes());
            }
        }
    }
    Ok(dir.join(format!("wf_{}_{:016x}.bin", CACHE_VERSION, state)))
}

#[inline]
pub fn decode_point(chunk: &[u8]) -> (u8, u8, u8, u8, u8, u8) {
    // Layout: rms_l, rms_r, up_l, down_l, up_r, down_r
    (chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5])
}

pub fn read_raw_points(cache_path: &Path, start: usize, count: usize) -> Result<Vec<u8>, String> {
    let mut file = File::open(cache_path).map_err(|e| format!("Cache open failed: {e}"))?;
    let pos = CACHE_HEADER_SIZE + (start as u64 * POINT_SIZE);
    file.seek(SeekFrom::Start(pos))
        .map_err(|e| format!("Cache seek failed: {e}"))?;
    let mut data = vec![0u8; count * POINT_SIZE as usize];
    file.read_exact(&mut data)
        .map_err(|e| format!("Cache read failed: {e}"))?;
    Ok(data)
}

pub fn probe(cache_path: &Path) -> usize {
    if !cache_path.exists() {
        return 0;
    }
    let mut file = match File::open(cache_path) {
        Ok(f) => f,
        Err(_) => return 0,
    };
    let mut header = [0u8; 8];
    if file.read_exact(&mut header).is_err() || &header[0..4] != b"WVFM" {
        let _ = fs::remove_file(cache_path);
        return 0;
    }
    let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
    if version != CACHE_VERSION {
        let _ = fs::remove_file(cache_path);
        return 0;
    }
    let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
    if file_size < CACHE_HEADER_SIZE {
        let _ = fs::remove_file(cache_path);
        return 0;
    }
    let data_size = file_size - CACHE_HEADER_SIZE;
    let max_data_size = TOTAL_WAVEFORM_POINTS as u64 * POINT_SIZE;
    let valid_size = CACHE_HEADER_SIZE + data_size.min(max_data_size);
    let cached_points = (data_size.min(max_data_size) / POINT_SIZE) as usize;

    if file_size > valid_size {
        drop(file);
        if OpenOptions::new()
            .write(true)
            .open(cache_path)
            .and_then(|f| f.set_len(valid_size))
            .is_err()
        {
            let _ = fs::remove_file(cache_path);
            return 0;
        }
    }
    cached_points
}

pub fn open_writer(cache_path: &Path) -> Option<BufWriter<File>> {
    let existed_before = cache_path.exists();

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(cache_path)
        .ok()?;

    let mut writer = BufWriter::new(file);

    if !existed_before {
        let _ = writer.write_all(b"WVFM");
        let _ = writer.write_all(&CACHE_VERSION.to_le_bytes());
        let _ = writer.flush();
    }

    Some(writer)
}
