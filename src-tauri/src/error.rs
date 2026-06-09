use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid video file: {0}")]
    InvalidVideo(String),

    #[error("FFmpeg error: {0}")]
    FFmpegError(String),

    #[error("FFprobe error: {0}")]
    FFprobeError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid segment: {0}")]
    InvalidSegment(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Export failed: {0}")]
    ExportError(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
