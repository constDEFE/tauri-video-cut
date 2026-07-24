use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;

use crate::utils::paths::app_temp_dir;

pub const CACHE_VERSION: u32 = 1;
pub const CACHE_HEADER_SIZE: u64 = 8; // magic(4) + version(4)
pub const POINT_SIZE: u64 = 6; // rms_l, rms_r, up_l, down_l, up_r, down_r

pub fn get_cache_path(
    video_path: &str,
    track_index: u32,
    target_rate: u32,
    points_per_event: usize,
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
    mix(&mut state, video_path.as_bytes());
    mix(&mut state, &track_index.to_le_bytes());
    mix(&mut state, &target_rate.to_le_bytes());
    mix(&mut state, &(points_per_event as u32).to_le_bytes());
    let duration_ms = (duration * 1000.0).round() as u64;
    mix(&mut state, &duration_ms.to_le_bytes());

    Ok(dir.join(format!("wf_{:016x}.bin", state)))
}

pub struct CacheState {
    pub cached_points: usize,
    pub cache_file: Option<BufWriter<File>>,
}

impl CacheState {
    pub fn open(cache_path: &PathBuf, _start_point: usize) -> Result<Self, String> {
        let mut cached_points = 0;

        if cache_path.exists() {
            if let Ok(mut file) = File::open(cache_path) {
                let mut header = [0u8; 8];
                if file.read_exact(&mut header).is_ok() && &header[0..4] == b"WVFM" {
                    let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);

                    if version == CACHE_VERSION {
                        let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
                        if file_size >= CACHE_HEADER_SIZE {
                            let data_size = file_size - CACHE_HEADER_SIZE;
                            cached_points = (data_size / POINT_SIZE) as usize;

                            let valid_size =
                                CACHE_HEADER_SIZE + (cached_points as u64 * POINT_SIZE);
                            if file_size > valid_size {
                                drop(file);
                                let truncated = OpenOptions::new()
                                    .write(true)
                                    .open(cache_path)
                                    .and_then(|f| f.set_len(valid_size));
                                if truncated.is_err() {
                                    let _ = fs::remove_file(cache_path);
                                    cached_points = 0;
                                }
                            }
                        }
                    } else {
                        drop(file);
                        let _ = fs::remove_file(cache_path);
                    }
                } else {
                    drop(file);
                    let _ = fs::remove_file(cache_path);
                }
            }
        }

        let mut cache_file = None;
        if let Ok(file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(cache_path)
        {
            let mut writer = BufWriter::new(file);
            if cached_points == 0 {
                let _ = writer.write_all(b"WVFM");
                let _ = writer.write_all(&CACHE_VERSION.to_le_bytes());
                let _ = writer.flush();
            }
            cache_file = Some(writer);
        }

        Ok(Self {
            cached_points,
            cache_file,
        })
    }
}
