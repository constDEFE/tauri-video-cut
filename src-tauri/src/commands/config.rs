use crate::config;
use crate::config::AppConfig;
use tauri::AppHandle;
use tracing::instrument;

#[instrument(skip(app, config), fields(theme = %config.theme), err(Debug))]
#[tauri::command]
pub async fn set_app_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    config::set_app_config(&app, &config)
        .await
        .map_err(|e| e.to_string())
}

#[instrument(skip(app, value), fields(key = %key), err(Debug))]
#[tauri::command]
pub async fn set_app_config_var(
    app: AppHandle,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    config::set_app_config_var(&app, &key, value)
        .await
        .map_err(|e| e.to_string())
}
