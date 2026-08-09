use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::services::ssh_client::{self, SshConfig};
use russh_sftp::client::SftpSession;
use russh_sftp::protocol::OpenFlags;

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub permissions: u32,
    pub modified: String,
}

async fn open_sftp(config: &SshConfig) -> Result<SftpSession, String> {
    let handle = ssh_client::connect(config).await?;
    let session = handle.lock().await;
    let channel = session
        .channel_open_session()
        .await
        .map_err(|e| format!("open channel failed: {e}"))?;
    channel
        .request_subsystem(true, "sftp")
        .await
        .map_err(|e| format!("request sftp failed: {e}"))?;
    SftpSession::new(channel.into_stream())
        .await
        .map_err(|e| format!("sftp session failed: {e}"))
}

fn entry_from_direntry(entry: &russh_sftp::client::fs::DirEntry) -> FileEntry {
    let name = entry.file_name();
    let meta = entry.metadata();
    let modified = meta
        .mtime
        .map(|t| {
            let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(t as i64, 0)
                .unwrap_or_default();
            dt.format("%Y-%m-%d %H:%M").to_string()
        })
        .unwrap_or_default();
    FileEntry {
        name,
        path: entry.path(),
        is_dir: meta.is_dir(),
        size: meta.len(),
        permissions: meta.permissions.unwrap_or(0),
        modified,
    }
}

pub async fn list(config: &SshConfig, path: &str) -> Result<Vec<FileEntry>, String> {
    let sftp = open_sftp(config).await?;
    let read_dir = sftp
        .read_dir(path)
        .await
        .map_err(|e| format!("read_dir failed: {e}"))?;
    let mut entries: Vec<FileEntry> = read_dir
        .map(|entry| entry_from_direntry(&entry))
        .collect();
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(entries)
}

pub async fn upload(
    config: &SshConfig,
    local: &str,
    remote: &str,
) -> Result<(), String> {
    let data = tokio::fs::read(local)
        .await
        .map_err(|e| format!("read local file failed: {e}"))?;
    let sftp = open_sftp(config).await?;
    let mut file = sftp
        .open_with_flags(
            remote,
            OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE,
        )
        .await
        .map_err(|e| format!("open remote file failed: {e}"))?;
    file.write_all(&data)
        .await
        .map_err(|e| format!("write failed: {e}"))?;
    file.sync_all()
        .await
        .map_err(|e| format!("sync failed: {e}"))?;
    file.shutdown()
        .await
        .map_err(|e| format!("shutdown failed: {e}"))
}

pub async fn download(
    config: &SshConfig,
    remote: &str,
    local: &str,
) -> Result<(), String> {
    let sftp = open_sftp(config).await?;
    let mut file = sftp
        .open(remote)
        .await
        .map_err(|e| format!("open remote file failed: {e}"))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .await
        .map_err(|e| format!("read failed: {e}"))?;
    tokio::fs::write(local, &data)
        .await
        .map_err(|e| format!("write local file failed: {e}"))
}

pub async fn delete(config: &SshConfig, path: &str) -> Result<(), String> {
    let sftp = open_sftp(config).await?;
    match sftp.metadata(path).await {
        Ok(meta) if meta.is_dir() => {
            sftp.remove_dir(path)
                .await
                .map_err(|e| format!("remove_dir failed: {e}"))
        }
        _ => sftp
            .remove_file(path)
            .await
            .map_err(|e| format!("remove_file failed: {e}")),
    }
}

pub async fn rename(
    config: &SshConfig,
    old_path: &str,
    new_path: &str,
) -> Result<(), String> {
    let sftp = open_sftp(config).await?;
    sftp.rename(old_path, new_path)
        .await
        .map_err(|e| format!("rename failed: {e}"))
}

pub async fn read_text(config: &SshConfig, path: &str) -> Result<String, String> {
    let sftp = open_sftp(config).await?;
    let mut file = sftp
        .open(path)
        .await
        .map_err(|e| format!("open failed: {e}"))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .await
        .map_err(|e| format!("read failed: {e}"))?;
    Ok(content)
}

pub async fn write_text(
    config: &SshConfig,
    path: &str,
    content: &str,
) -> Result<(), String> {
    let sftp = open_sftp(config).await?;
    let mut file = sftp
        .open_with_flags(
            path,
            OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE,
        )
        .await
        .map_err(|e| format!("open failed: {e}"))?;
    file.write_all(content.as_bytes())
        .await
        .map_err(|e| format!("write failed: {e}"))?;
    file.sync_all()
        .await
        .map_err(|e| format!("sync failed: {e}"))?;
    file.shutdown()
        .await
        .map_err(|e| format!("shutdown failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ssh_client::SshAuth;

    fn test_config() -> Option<SshConfig> {
        let pw = std::env::var("TX_TEST_PASSWORD").ok()?;
        Some(SshConfig {
            host: std::env::var("TX_TEST_HOST").unwrap_or_else(|_| "47.100.33.169".to_string()),
            port: std::env::var("TX_TEST_PORT")
                .unwrap_or_else(|_| "22".to_string())
                .parse()
                .unwrap_or(22),
            username: std::env::var("TX_TEST_USER").unwrap_or_else(|_| "root".to_string()),
            auth: SshAuth::Password { password: pw },
        })
    }

    #[tokio::test]
    async fn test_sftp_list_root() {
        let Some(config) = test_config() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let entries = list(&config, "/root").await.expect("list failed");
        assert!(!entries.is_empty(), "should list some entries in /root");
        assert!(entries.iter().any(|e| e.is_dir), "should contain dirs");
        eprintln!(
            "first entries: {:?}",
            entries.iter().take(5).map(|e| (&e.name, e.is_dir)).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_sftp_upload_download_roundtrip() {
        let Some(config) = test_config() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let dir = std::env::temp_dir();
        let remote = format!("/tmp/tianxuan_sftp_test_{}.txt", uuid::Uuid::new_v4());
        let local = dir.join("tianxuan_sftp_test.txt");

        write_text(&config, &remote, "hello sftp roundtrip").await.expect("write_text");
        let content = read_text(&config, &remote).await.expect("read_text");
        assert_eq!(content, "hello sftp roundtrip");
        download(&config, &remote, local.to_str().unwrap()).await.expect("download");
        let local_content = tokio::fs::read_to_string(&local).await.unwrap();
        assert_eq!(local_content, "hello sftp roundtrip");
        let remote2 = format!("{remote}.2");
        upload(&config, local.to_str().unwrap(), &remote2).await.expect("upload");
        assert_eq!(read_text(&config, &remote2).await.unwrap(), "hello sftp roundtrip");
        delete(&config, &remote).await.expect("delete remote");
        delete(&config, &remote2).await.expect("delete remote2");
        let _ = tokio::fs::remove_file(&local).await;
    }

    #[tokio::test]
    async fn test_sftp_rename() {
        let Some(config) = test_config() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let a = format!("/tmp/tianxuan_rename_{}.txt", uuid::Uuid::new_v4());
        let b = format!("{a}.renamed");
        write_text(&config, &a, "rename me").await.expect("write");
        rename(&config, &a, &b).await.expect("rename");
        let content = read_text(&config, &b).await.expect("read renamed");
        assert_eq!(content, "rename me");
        delete(&config, &b).await.expect("cleanup");
    }
}

