use serde::{Deserialize, Serialize};

pub const TOTAL_WAVEFORM_POINTS: usize = 1_920;
pub const DEFAULT_POINTS_PER_EVENT: usize = 96;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamWaveformRequest {
    pub video_path: String,
    pub track_index: u32,
    pub duration: f64,
    #[serde(default)]
    pub target_rate: Option<u32>,
    #[serde(default)]
    pub audio_tracks_sample_rate: Option<u32>,
    #[serde(default)]
    pub points_per_event: Option<usize>,
    #[serde(default)]
    pub resume_from_point: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartWaveformResponse {
    pub job_id: String,
    pub total_points: usize,
    pub points_per_event: usize,
    pub event_count: usize,
    pub target_rate: u32,
    pub cached_data: Option<WaveformChunkEvent>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveformChunkEvent {
    pub job_id: String,
    pub track_index: u32,
    pub chunk_index: usize,
    pub point_offset: usize,
    pub point_count: usize,
    pub total_points: usize,
    pub progress: f32,
    pub points_per_event: usize,

    pub left_rms: Vec<u8>,
    pub right_rms: Vec<u8>,

    pub left_peak_up: Vec<u8>,
    pub left_peak_down: Vec<u8>,
    pub right_peak_up: Vec<u8>,
    pub right_peak_down: Vec<u8>,

    pub chunk_max_peak: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveformFinishedEvent {
    pub job_id: String,
    pub track_index: u32,
    pub total_points: usize,
    pub decoded_frames: u64,
    pub expected_frames: u64,
    pub target_rate: u32,
    pub max_left_peak: u8,
    pub max_right_peak: u8,
    pub display_gain: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveformErrorEvent {
    pub job_id: String,
    pub track_index: u32,
    pub message: String,
}
