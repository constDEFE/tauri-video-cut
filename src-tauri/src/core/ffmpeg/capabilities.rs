use crate::core::ProcessManager;
use crate::error::{AppError, Result};
use crate::logger::{log_debug, log_error, log_info, log_warn};
use crate::utils::cmd::new_command;
use std::collections::HashSet;
use std::path::Path;
use std::process::Stdio;
use std::sync::Mutex;

static HW_CAPABILITIES: Mutex<Option<HwCapabilities>> = Mutex::new(None);

#[derive(Debug, Clone)]
pub struct HwCapabilities {
    pub encoders: HashSet<String>,
}

impl HwCapabilities {
    pub async fn detect(ffmpeg_path: &Path, process_manager: &ProcessManager) -> Result<Self> {
        let encoders = detect_encoders(ffmpeg_path, process_manager).await?;
        Ok(Self { encoders })
    }

    pub fn has_encoder(&self, encoder: &str) -> bool {
        self.encoders.contains(encoder)
    }
}

pub async fn get_hw_capabilities(
    ffmpeg_path: &Path,
    process_manager: &ProcessManager,
) -> Result<HwCapabilities> {
    {
        let cache = HW_CAPABILITIES.lock().unwrap();
        if let Some(caps) = cache.as_ref() {
            log_debug!("Returning cached HW encoder capabilities");

            return Ok(caps.clone());
        }
    }

    log_info!("Detecting HW encoder capabilities");

    let caps = HwCapabilities::detect(ffmpeg_path, process_manager).await?;
    let mut cache = HW_CAPABILITIES.lock().unwrap();
    *cache = Some(caps.clone());

    Ok(caps)
}

async fn detect_encoders(
    ffmpeg_path: &Path,
    process_manager: &ProcessManager,
) -> Result<HashSet<String>> {
    let child = new_command(ffmpeg_path)
        .args(&["-encoders", "-hide_banner"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            log_error!(error = %e, "Failed to spawn ffmpeg for encoder detection");

            AppError::FFmpegError(format!(
                "Failed to spawn ffmpeg for encoder detection: {}",
                e
            ))
        })?;

    process_manager.attach(&child)?;

    let output = child.wait_with_output().await.map_err(|e| {
        log_error!(error = %e, "Failed waiting for ffmpeg encoder detection");

        AppError::FFmpegError(format!("Failed to detect encoders: {}", e))
    })?;

    if !output.status.success() {
        log_error!(status = %output.status, stderr = %String::from_utf8_lossy(&output.stderr), "ffmpeg -encoders returned non-zero");

        return Err(AppError::FFmpegError(
            "Failed to run ffmpeg -encoders".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut encoders = HashSet::new();

    for line in stdout.lines() {
        if line.contains('=') || line.contains("---") {
            continue;
        }

        if line.starts_with(" V") || line.starts_with(" A") {
            if line.len() > 8 {
                let remainder = &line[8..];
                let encoder_name = remainder.split_whitespace().next();
                if let Some(name) = encoder_name {
                    encoders.insert(name.to_string());
                }
            }
        }
    }

    Ok(encoders)
}

pub fn get_encoder_fallback_chain(codec: &str, caps: &HwCapabilities) -> Vec<String> {
    let candidates = match codec {
        "h264" => vec!["h264_nvenc", "libx264"],
        "hevc" | "h265" => vec!["hevc_nvenc", "libx265"],
        "av1" => vec!["av1_nvenc", "libsvtav1"],
        "vp9" => vec!["libvpx-vp9"],
        "vp8" => vec!["libvpx"],
        "mpeg4" | "mpeg2video" => vec!["libx264"],
        _ => vec!["libx264"],
    };

    let mut chain = Vec::new();
    for encoder in candidates {
        if caps.has_encoder(encoder) {
            chain.push(encoder.to_string());
        }
    }

    if chain.is_empty() {
        log_warn!(codec = %codec, "No matching encoders found, falling back to libx264");

        chain.push("libx264".to_string());
    }

    chain
}
