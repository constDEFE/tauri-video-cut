use crate::config::model::AppConfig;
use crate::logger::log_debug;
use crate::utils::atomic;
use anyhow::{Context, Result};
use tauri::Manager;
use tokio::sync::Mutex;

static CONFIG_LOCK: Mutex<()> = Mutex::const_new(());

pub fn load_config(app_handle: &tauri::AppHandle) -> Result<AppConfig> {
    let config_path = app_handle
        .path()
        .resolve("config.json", tauri::path::BaseDirectory::Resource)?;

    log_debug!(path = ?config_path, exists = config_path.exists(), "Loading app config");

    if config_path.exists() {
        let content =
            std::fs::read_to_string(&config_path).context("Failed to read config file")?;
        let config: AppConfig =
            serde_json::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

pub async fn set_app_config(app_handle: &tauri::AppHandle, config: &AppConfig) -> Result<()> {
    let _guard = CONFIG_LOCK.lock().await;

    log_debug!("Persisting app config");

    let config_path = app_handle
        .path()
        .resolve("config.json", tauri::path::BaseDirectory::Resource)?;
    let content = serde_json::to_string_pretty(config).context("Failed to serialize config")?;

    atomic::atomic_write(&config_path, &content).await?;

    Ok(())
}

pub async fn set_app_config_var(
    app_handle: &tauri::AppHandle,
    key: &str,
    value: serde_json::Value,
) -> Result<()> {
    let _guard = CONFIG_LOCK.lock().await;

    log_debug!(key = %key, "Updating config variable");

    let config_path = app_handle
        .path()
        .resolve("config.json", tauri::path::BaseDirectory::Resource)?;
    let content = if config_path.exists() {
        let existing =
            std::fs::read_to_string(&config_path).context("Failed to read config file")?;
        serde_json::from_str::<serde_json::Value>(&existing)
            .context("Failed to parse config file")?
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };

    let mut config = content.as_object().cloned().unwrap_or_default();
    config.insert(key.to_string(), value);

    let content = serde_json::to_string_pretty(&config).context("Failed to serialize config")?;

    atomic::atomic_write(&config_path, &content).await?;

    Ok(())
}
