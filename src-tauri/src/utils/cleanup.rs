use crate::logger::log_info;
use crate::utils::paths::app_temp_dir;
use std::ffi::OsStr;
use std::fs;
use std::time::{Duration, SystemTime};

pub fn cleanup_orphaned_temp_segments() {
    let temp_dir = app_temp_dir().join("temp_segments");

    if !temp_dir.exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(&temp_dir) {
        let mut removed_count: u8 = 0;

        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Ok(_) = fs::remove_file(&path) {
                        removed_count += 1;
                    }
                }
            }
        }

        log_info!(removed_count, "Cleaned up orphaned temp segments");
    }
}

pub fn cleanup_old_waveforms() {
    let dir = app_temp_dir().join("cache").join("waveforms");

    if !dir.exists() {
        return;
    }

    let cutoff = SystemTime::now() - Duration::from_secs(30 * 24 * 3600);

    if let Ok(entries) = fs::read_dir(&dir) {
        let mut removed_count: u32 = 0;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.extension() == Some(OsStr::new("bin")) {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified < cutoff {
                            if let Ok(_) = fs::remove_file(&path) {
                                removed_count += 1;
                            }
                        }
                    }
                }
            }
        }

        log_info!(
            removed_count,
            cutoff_days = 30,
            "Cleaned up old waveform caches"
        );
    }
}
