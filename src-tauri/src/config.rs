use crate::types::config::AppConfig;
use anyhow::{Context, Result};
use tauri::Manager;

pub fn load_config(app_handle: &tauri::AppHandle) -> Result<AppConfig> {
    let config_path = app_handle
        .path()
        .resolve("config.json", tauri::path::BaseDirectory::Resource)?;

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        let config: AppConfig = serde_json::from_str(&content)
            .context("Failed to parse config file")?;
        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

pub async fn set_app_config(app_handle: &tauri::AppHandle, config: &AppConfig) -> Result<()> {
    let config_path = app_handle
        .path()
        .resolve("config.json", tauri::path::BaseDirectory::Resource)?;
    let content = serde_json::to_string_pretty(config)
        .context("Failed to serialize config")?;

    tokio::fs::write(&config_path, content)
        .await
        .context("Failed to write config file")?;

    Ok(())
}

pub async fn set_app_config_var(
    app_handle: &tauri::AppHandle,
    key: &str,
    value: serde_json::Value,
) -> Result<()> {
    let config_path = app_handle
        .path()
        .resolve("config.json", tauri::path::BaseDirectory::Resource)?;
    let content = if config_path.exists() {
        let existing = std::fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        serde_json::from_str::<serde_json::Value>(&existing)
            .context("Failed to parse config file")?
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };

    let mut config = content.as_object().cloned().unwrap_or_default();
    config.insert(key.to_string(), value);

    let content = serde_json::to_string_pretty(&config)
        .context("Failed to serialize config")?;

    tokio::fs::write(&config_path, content)
        .await
        .context("Failed to write config file")?;

    Ok(())
}
