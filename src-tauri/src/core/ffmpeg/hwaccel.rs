use crate::core::process::ProcessManager;
use crate::error::{AppError, Result};
use crate::logger::log_error;
use std::collections::HashSet;
use std::path::Path;
use std::process::Stdio;
use std::sync::Mutex;
use tokio::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn new_command(program: &Path) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

static HW_CAPABILITIES: Mutex<Option<HwCapabilities>> = Mutex::new(None);

#[derive(Debug, Clone)]
pub struct HwCapabilities {
    pub encoders: HashSet<String>,
    pub decoders: HashSet<String>,
    pub hwaccels: HashSet<String>,
}

unsafe impl Send for HwCapabilities {}

impl HwCapabilities {
    pub async fn detect(ffmpeg_path: &Path, process_manager: &ProcessManager) -> Result<Self> {
        let encoders = detect_encoders(ffmpeg_path, process_manager).await?;
        let decoders = detect_decoders(ffmpeg_path, process_manager).await?;
        let hwaccels = detect_hwaccels(ffmpeg_path, process_manager).await?;

        Ok(Self {
            encoders,
            decoders,
            hwaccels,
        })
    }

    pub fn has_encoder(&self, encoder: &str) -> bool {
        self.encoders.contains(encoder)
    }

    pub fn has_decoder(&self, decoder: &str) -> bool {
        self.decoders.contains(decoder)
    }

    pub fn has_hwaccel(&self, hwaccel: &str) -> bool {
        self.hwaccels.contains(hwaccel)
    }
}

pub async fn get_hw_capabilities(
    ffmpeg_path: &Path,
    process_manager: &ProcessManager,
) -> Result<HwCapabilities> {
    {
        let cache = HW_CAPABILITIES.lock().unwrap();
        if let Some(caps) = cache.as_ref() {
            return Ok(caps.clone());
        }
    }

    let caps = HwCapabilities::detect(ffmpeg_path, process_manager).await?;

    {
        let mut cache = HW_CAPABILITIES.lock().unwrap();
        *cache = Some(caps.clone());
    }

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
            AppError::FFmpegError(format!(
                "Failed to spawn ffmpeg for encoder detection: {}",
                e
            ))
        })?;

    // Attach ffmpeg to ProcessManager for clean shutdown
    process_manager.attach(&child)?;

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| AppError::FFmpegError(format!("Failed to detect encoders: {}", e)))?;

    if !output.status.success() {
        log_error("Failed to run ffmpeg -encoders");
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

async fn detect_decoders(
    ffmpeg_path: &Path,
    process_manager: &ProcessManager,
) -> Result<HashSet<String>> {
    let child = new_command(ffmpeg_path)
        .args(&["-decoders", "-hide_banner"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            AppError::FFmpegError(format!(
                "Failed to spawn ffmpeg for decoder detection: {}",
                e
            ))
        })?;

    // Attach ffmpeg to ProcessManager for clean shutdown
    process_manager.attach(&child)?;

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| AppError::FFmpegError(format!("Failed to detect decoders: {}", e)))?;

    if !output.status.success() {
        log_error("Failed to run ffmpeg -decoders");
        return Err(AppError::FFmpegError(
            "Failed to run ffmpeg -decoders".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut decoders = HashSet::new();

    for line in stdout.lines() {
        if line.contains('=') || line.contains("---") {
            continue;
        }

        if line.starts_with(" V") || line.starts_with(" A") {
            if line.len() > 8 {
                let remainder = &line[8..];
                let decoder_name = remainder.split_whitespace().next();
                if let Some(name) = decoder_name {
                    decoders.insert(name.to_string());
                }
            }
        }
    }

    Ok(decoders)
}

async fn detect_hwaccels(
    ffmpeg_path: &Path,
    process_manager: &ProcessManager,
) -> Result<HashSet<String>> {
    let child = new_command(ffmpeg_path)
        .args(&["-hwaccels", "-hide_banner"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            AppError::FFmpegError(format!(
                "Failed to spawn ffmpeg for hwaccel detection: {}",
                e
            ))
        })?;

    // Attach ffmpeg to ProcessManager for clean shutdown
    process_manager.attach(&child)?;

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| AppError::FFmpegError(format!("Failed to detect hwaccels: {}", e)))?;

    if !output.status.success() {
        log_error("Failed to run ffmpeg -hwaccels");
        return Err(AppError::FFmpegError(
            "Failed to run ffmpeg -hwaccels".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut hwaccels = HashSet::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("Hardware") {
            hwaccels.insert(trimmed.to_string());
        }
    }

    Ok(hwaccels)
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwAccelType {
    Cuda,
    D3d11va,
    Dxva2,
    None,
}

pub fn select_hwaccel(caps: &HwCapabilities) -> HwAccelType {
    if caps.has_hwaccel("cuda") {
        HwAccelType::Cuda
    } else if caps.has_hwaccel("d3d11va") {
        HwAccelType::D3d11va
    } else if caps.has_hwaccel("dxva2") {
        HwAccelType::Dxva2
    } else {
        HwAccelType::None
    }
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
        chain.push("libx264".to_string());
    }

    chain
}

pub fn get_decoder_fallback_chain(codec: &str, caps: &HwCapabilities) -> Vec<Option<String>> {
    let candidates = match codec {
        "h264" => vec!["h264_cuvid"],
        "hevc" | "h265" => vec!["hevc_cuvid"],
        "av1" => vec!["av1_cuvid", "libdav1d"],
        "vp9" => vec!["vp9_cuvid"],
        _ => vec![],
    };

    let mut chain = Vec::new();

    for decoder in candidates {
        if caps.has_decoder(decoder) {
            chain.push(Some(decoder.to_string()));
        }
    }

    chain.push(None);

    chain
}

pub fn build_hwaccel_args(hwaccel: &HwAccelType) -> Vec<String> {
    match hwaccel {
        HwAccelType::Cuda => vec![
            "-hwaccel".to_string(),
            "cuda".to_string(),
            "-hwaccel_output_format".to_string(),
            "cuda".to_string(),
        ],
        HwAccelType::D3d11va => vec!["-hwaccel".to_string(), "d3d11va".to_string()],
        HwAccelType::Dxva2 => vec!["-hwaccel".to_string(), "dxva2".to_string()],
        HwAccelType::None => vec![],
    }
}
