use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub segments: Option<Vec<PrunedSegment>>,
    #[serde(default)]
    pub audio_tracks: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrunedSegment {
    pub id: String,
    pub start: f64,
    pub end: f64,
}

impl Session {
    pub fn blank() -> Self {
        Self {
            file_path: None,
            segments: None,
            audio_tracks: None,
        }
    }

    pub fn is_blank(&self) -> bool {
        self.file_path.is_none()
            && self.segments.as_deref().map_or(true, |s| s.is_empty())
            && self.audio_tracks.as_deref().map_or(true, |t| t.is_empty())
    }

    pub fn validate(&self) -> Result<()> {
        if self.is_blank() {
            return Ok(());
        }

        let file_path = self
            .file_path
            .as_deref()
            .ok_or_else(|| AppError::Other("session: missing file_path".into()))?;

        if !std::path::Path::new(file_path).exists() {
            return Err(AppError::Other(format!(
                "session: file does not exist: {}",
                file_path
            )));
        }

        if let Some(segments) = &self.segments {
            for seg in segments {
                if seg.start < 0.0 || seg.end <= seg.start {
                    return Err(AppError::Other(format!(
                        "session: invalid segment range [{}, {}]",
                        seg.start, seg.end
                    )));
                }
            }
        }

        if let Some(tracks) = &self.audio_tracks {
            for track in tracks {
                if *track < 0 {
                    return Err(AppError::Other("session: negative audio track id".into()));
                }
            }
        }
        Ok(())
    }
}
