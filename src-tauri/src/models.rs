use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTrack {
    pub index: usize,
    pub track_id: Option<u32>,
    pub codec: String,
    pub channels: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub video_codec: String,
    pub bitrate: u64,
    pub fps: f64,
    pub audio_tracks: Vec<AudioTrack>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SegmentExportRequest {
    pub start: f64,
    pub end: f64,
    pub audio_tracks: Vec<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportRequest {
    pub video_path: String,
    pub output_folder: String,
    pub file_prefix: String,
    pub smart_cut: bool,
    pub segments: Vec<SegmentExportRequest>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportProgress {
    pub current_segment: usize,
    pub total_segments: usize,
    pub current_segment_progress: f64,
    pub eta_seconds: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportResult {
    pub success: bool,
    pub output_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default, rename = "outputFolder")]
    pub output_folder: String
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            output_folder: String::new()
        }
    }
}
