use crate::error::{AppError, Result};
use crate::logger::{log_debug, log_error, log_warn};
use crate::session::model::Session;
use crate::utils::atomic;
use std::path::PathBuf;
use tauri::Manager;

pub fn load_session(app_handle: &tauri::AppHandle) -> Session {
    let session_path = match get_session_path(app_handle) {
        Ok(p) => p,
        Err(e) => {
            log_error!(error = %e, "Failed to resolve session path");

            return Session::blank();
        }
    };

    if !session_path.exists() {
        if let Err(e) = save_session_sync(app_handle, &Session::blank()) {
            log_error!(error = %e, "Failed to persist blank session");
        }

        return Session::blank();
    }

    let content = match std::fs::read_to_string(&session_path) {
        Ok(c) => c,
        Err(e) => {
            log_error!(path = ?session_path, error = %e, "Failed to read session file");

            return Session::blank();
        }
    };

    let session: Session = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            log_warn!(path = ?session_path, error = %e, "Session file corrupt; resetting to blank");

            if let Err(e) = save_session_sync(app_handle, &Session::blank()) {
                log_error!(error = %e, "Failed to persist blank session");
            }

            return Session::blank();
        }
    };

    if let Err(e) = session.validate() {
        log_warn!(error = %e, "Session validation failed; resetting to blank");

        if let Err(e) = save_session_sync(app_handle, &Session::blank()) {
            log_error!(error = %e, "Failed to persist blank session");
        }

        return Session::blank();
    }

    log_debug!(
        has_file = session.file_path.is_some(),
        segments = session.segments.as_ref().map_or(0, |s| s.len()),
        "Session loaded successfully"
    );

    session
}

pub async fn save_session(app_handle: &tauri::AppHandle, session: &Session) -> Result<()> {
    let session_path = get_session_path(app_handle)?;
    let content = serde_json::to_string_pretty(session).map_err(|e| AppError::JsonError(e))?;

    atomic::atomic_write(&session_path, &content)
        .await
        .map_err(|e| AppError::Other(e.to_string()))?;

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
