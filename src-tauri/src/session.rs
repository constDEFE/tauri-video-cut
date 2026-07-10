use crate::{
    error::{AppError, Result},
    logger::{log_error, log_warn},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

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

    /// session is blank if all fields None/empty
    pub fn is_blank(&self) -> bool {
        self.file_path.is_none()
            && self.segments.as_ref().is_some_and(|s| s.is_empty())
            && self.audio_tracks.as_ref().is_some_and(|t| t.is_empty())
    }

    /// Blank sessions are always valid
    pub fn validate(&self) -> Result<()> {
        // Blank session is valid
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

pub async fn atomic_write(path: &PathBuf, content: &str) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    tokio::fs::write(&tmp, content)
        .await
        .map_err(|e| AppError::IoError(e))?;
    match tokio::fs::rename(&tmp, path).await {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = tokio::fs::remove_file(&tmp).await;
            Err(AppError::IoError(e))
        }
    }
}

pub fn load_session(app_handle: &tauri::AppHandle) -> Session {
    let session_path = match get_session_path(app_handle) {
        Ok(p) => p,
        Err(e) => {
            log_error(&format!("Failed to resolve session path: {}", e));
            return Session::blank();
        }
    };

    if !session_path.exists() {
        if let Err(e) = save_session_sync(app_handle, &Session::blank()) {
            log_error(&format!("Failed to create blank session: {}", e));
        }

        return Session::blank();
    }

    let content = match std::fs::read_to_string(&session_path) {
        Ok(c) => c,
        Err(e) => {
            log_error(&format!("Failed to read session file: {}", e));
            return Session::blank();
        }
    };

    let session: Session = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            log_error(&format!(
                "Failed to parse session file: {}, creating blank",
                e
            ));
            if let Err(e) = save_session_sync(app_handle, &Session::blank()) {
                log_error(&format!("Failed to create blank session: {}", e));
            }
            return Session::blank();
        }
    };

    if let Err(e) = session.validate() {
        log_warn(&format!("Session validation failed: {}, creating blank", e));
        if let Err(e) = save_session_sync(app_handle, &Session::blank()) {
            log_error(&format!("Failed to create blank session: {}", e));
        }
        return Session::blank();
    }

    return session;
}

pub async fn save_session(app_handle: &tauri::AppHandle, session: &Session) -> Result<()> {
    let session_path = get_session_path(app_handle)?;
    let content = serde_json::to_string_pretty(session).map_err(|e| AppError::JsonError(e))?;

    atomic_write(&session_path, &content).await?;

    Ok(())
}

pub fn get_session_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    app_handle
        .path()
        .resolve("session.json", tauri::path::BaseDirectory::Resource)
        .map_err(|e| AppError::Other(format!("Failed to resolve session path: {}", e)))
}

pub fn save_session_sync(app_handle: &tauri::AppHandle, session: &Session) -> Result<()> {
    let session_path = get_session_path(app_handle)?;
    let content = serde_json::to_string_pretty(session).map_err(|e| AppError::JsonError(e))?;
    let tmp = session_path.with_extension("json.tmp");

    std::fs::write(&tmp, content).map_err(|e| AppError::IoError(e))?;

    match std::fs::rename(&tmp, &session_path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(AppError::IoError(e))
        }
    }
}

pub async fn blank_session(app_handle: &tauri::AppHandle) -> Result<()> {
    save_session(app_handle, &Session::blank()).await
}
