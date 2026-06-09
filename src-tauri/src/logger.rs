use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

struct FileLogger;

impl log::Log for FileLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let level = match record.level() {
            log::Level::Error => "ERROR",
            log::Level::Warn => "WARN",
            log::Level::Info => "INFO",
            log::Level::Debug => "DEBUG",
            log::Level::Trace => "TRACE",
        };
        write_log(level, record.args().to_string().as_str());
    }

    fn flush(&self) {
        if let Ok(mut guard) = LOG_FILE.lock() {
            if let Some(file) = guard.as_mut() {
                let _ = file.flush();
            }
        }
    }
}

pub fn init() {
    let log_dir = std::env::temp_dir().join("io.github.constdefe.tauri-video-cut").join("logs");
    if !log_dir.exists() {
        let _ = fs::create_dir_all(&log_dir);
    }

    rotate_logs(&log_dir);

    let timestamp = chrono::Local::now().format("%Y-%m-%d");
    let log_path = log_dir.join(format!("videocut-{}.log", timestamp));

    if let Ok(file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        *LOG_FILE.lock().unwrap() = Some(file);
        let _ = log::set_logger(&FileLogger).map(|()| log::set_max_level(log::LevelFilter::Trace));
        log_info("VideoCut started");
    }
}

fn rotate_logs(log_dir: &PathBuf) {
    if let Ok(entries) = fs::read_dir(log_dir) {
        let mut log_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "log")
                    .unwrap_or(false)
            })
            .collect();

        log_files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

        if log_files.len() >= 10 {
            for entry in log_files.iter().take(log_files.len() - 9) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

fn write_log(level: &str, message: &str) {
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(file) = guard.as_mut() {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(file, "[{}] {} - {}", timestamp, level, message);
        }
    }
}

pub fn log_info(message: &str) {
    write_log("INFO", message);
}

pub fn log_warn(message: &str) {
    write_log("WARN", message);
}

pub fn log_error(message: &str) {
    write_log("ERROR", message);
}
