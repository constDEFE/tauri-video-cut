use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;

pub async fn atomic_write(path: &PathBuf, content: &str) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, content)
        .await
        .context("Failed to write temp file")?;
    match fs::rename(&tmp, path).await {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&tmp).await;
            Err(e).context("Failed to atomically rename")
        }
    }
}
