use anyhow::Result;
use std::time::SystemTime;

pub async fn get_modified(path: &str) -> Result<u128> {
    Ok(tokio::fs::metadata(path)
        .await?
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis())
}
