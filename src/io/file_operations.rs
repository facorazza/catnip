use anyhow::Result;
use std::path::Path;
use tokio::fs;

pub async fn read_file_safe(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))
}

pub async fn write_file_safe(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", path.display(), e))
}
