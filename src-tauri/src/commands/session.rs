use crate::session;
use crate::session::Session;
use tauri::AppHandle;
use tracing::instrument;

#[instrument(
    skip(app, session),
    fields(
        has_file = session.file_path.is_some(),
        segments = session.segments.as_ref().map_or(0, |s| s.len())
    ),
    err(Debug)
)]
#[tauri::command]
pub async fn set_session(app: AppHandle, session: Session) -> Result<(), String> {
    session::save_session(&app, &session)
        .await
        .map_err(|e| e.to_string())
}
