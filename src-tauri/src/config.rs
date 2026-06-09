use crate::error::{AppError, Result};
use crate::models::AppConfig;
use crate::logger;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILENAME: &str = "config.json";

pub fn get_config_path() -> Result<PathBuf> {
    let exe_path = std::env::current_exe()
        .map_err(|e| AppError::ConfigError(format!("Failed to get executable path: {}", e)))?;

    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| AppError::ConfigError("Failed to get executable directory".to_string()))?;

    Ok(exe_dir.join(CONFIG_FILENAME))
}

fn get_system_theme() -> String {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        let output = Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                "/v",
                "AppsUseLightTheme",
            ])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("0x0") {
                return "dark".to_string();
            } else if stdout.contains("0x1") {
                return "light".to_string();
            }
        }
    }

    "dark".to_string()
}

fn validate_theme(theme: &str) -> String {
    match theme {
        "dark" | "light" => theme.to_string(),
        _ => get_system_theme(),
    }
}

fn validate_output_folder(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }

    let path_obj = Path::new(path);

    if !path_obj.is_absolute() {
        return String::new();
    }

    for component in path_obj.components() {
        let comp_str = component.as_os_str().to_string_lossy();

        let invalid_chars = ['<', '>', '"', '|', '?', '*'];
        if comp_str.chars().any(|c| invalid_chars.contains(&c)) {
            return String::new();
        }
    }

    path.to_string()
}

pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;

    let (mut config, needs_validation) = if !config_path.exists() {
        logger::log_info("Config file not found, creating default");
        (AppConfig::default(), true)
    } else {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| AppError::ConfigError(format!("Failed to read config: {}", e)))?;

        match serde_json::from_str::<AppConfig>(&content) {
            Ok(cfg) => {
                logger::log_info(&format!("Config loaded from {}", config_path.display()));
                (cfg, true)
            }
            Err(_) => {
                logger::log_warn("Invalid config file, using defaults");
                (AppConfig::default(), true)
            }
        }
    };

    if needs_validation {
        let original_theme = config.theme.clone();
        let original_output_folder = config.output_folder.clone();

        config.theme = validate_theme(&config.theme);
        config.output_folder = validate_output_folder(&config.output_folder);

        if !config_path.exists()
            || original_theme != config.theme
            || original_output_folder != config.output_folder
        {
            save_config(&config)?;
        }
    }

    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_path()?;

    let content = serde_json::to_string_pretty(config)?;

    fs::write(&config_path, content)
        .map_err(|e| AppError::ConfigError(format!("Failed to write config: {}", e)))?;

    logger::log_info(&format!("Config saved to {}", config_path.display()));

    Ok(())
}

pub fn set_app_config(config: &AppConfig) -> Result<()> {
    logger::log_info("Updating app config");
    let mut validated = config.clone();
    validated.theme = validate_theme(&config.theme);
    validated.output_folder = validate_output_folder(&config.output_folder);
    save_config(&validated)
}

pub fn set_app_config_var(key: &str, value: serde_json::Value) -> Result<()> {
    logger::log_info(&format!("Updating config key: {}", key));
    let mut config = load_config()?;
    let mut json_value = serde_json::to_value(&config)?;

    if let Some(obj) = json_value.as_object_mut() {
        obj.insert(key.to_string(), value);
        config = serde_json::from_value(json_value)?;

        config.theme = validate_theme(&config.theme);
        config.output_folder = validate_output_folder(&config.output_folder);

        save_config(&config)?;
    }

    Ok(())
}
