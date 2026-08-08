use std::fs;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::core::waveform::model::TOTAL_WAVEFORM_POINTS;
use crate::utils::fsx::{create_staging, open_shared_read};
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

pub fn part_path_for(cache_path: &Path, job_id: &str) -> PathBuf {
    let mut name = cache_path.file_name().unwrap_or_default().to_os_string();
    name.push(".");
    name.push(std::process::id().to_string());
    name.push("-");
    name.push(job_id);
    name.push(".part");
    cache_path.with_file_name(name)
}

#[inline]
pub fn decode_point(chunk: &[u8]) -> (u8, u8, u8, u8, u8, u8) {
    // Layout: rms_l, rms_r, up_l, down_l, up_r, down_r
    (chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5])
}

pub fn read_raw_points(cache_path: &Path, start: usize, count: usize) -> Result<Vec<u8>, String> {
    let mut file = open_shared_read(cache_path)
        .ok_or_else(|| format!("Cache open failed: {:?}", cache_path))?;
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
    let mut file = match open_shared_read(cache_path) {
        Some(f) => f,
        None => return 0,
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
    (data_size.min(max_data_size) / POINT_SIZE) as usize
}

pub struct CacheWriter {
    part_path: PathBuf,
    cache_path: PathBuf,
    writer: Option<BufWriter<std::fs::File>>,
    points_written: usize,
    published: bool,
}

impl CacheWriter {
    pub fn create(cache_path: &Path, job_id: &str, prefix: &[u8]) -> Option<Self> {
        let part_path = part_path_for(cache_path, job_id);
        let mut writer = create_staging(&part_path)?;
        writer.write_all(b"WVFM").ok()?;
        writer.write_all(&CACHE_VERSION.to_le_bytes()).ok()?;
        writer.write_all(prefix).ok()?;
        writer.flush().ok()?;
        Some(Self {
            part_path,
            cache_path: cache_path.to_path_buf(),
            writer: Some(writer),
            points_written: prefix.len() / POINT_SIZE as usize,
            published: false,
        })
    }

    pub fn write_batch(&mut self, batch: &[u8]) -> std::io::Result<()> {
        if let Some(w) = &mut self.writer {
            w.write_all(batch)?;
            w.flush()?;
            self.points_written += batch.len() / POINT_SIZE as usize;
        }
        Ok(())
    }

    pub fn publish(&mut self) {
        if self.published {
            return;
        }
        if let Some(mut w) = self.writer.take() {
            let _ = w.flush();
            drop(w);
        }

        let existing = probe(&self.cache_path);
        if self.points_written > existing {
            if std::fs::rename(&self.part_path, &self.cache_path).is_err() {
                let _ = std::fs::remove_file(&self.cache_path);
                if std::fs::rename(&self.part_path, &self.cache_path).is_err() {
                    let _ = std::fs::remove_file(&self.part_path);
                }
            }
        } else {
            let _ = std::fs::remove_file(&self.part_path);
        }
        self.published = true;
    }
}

impl Drop for CacheWriter {
    fn drop(&mut self) {
        if !self.published {
            self.publish();
        }
    }
}
