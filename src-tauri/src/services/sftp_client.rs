use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::models::Host;
use crate::services::ssh_client::connect_with_password;
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

async fn open_sftp(host: &Host, password: &str) -> Result<SftpSession, String> {
    let session = connect_with_password(&host.address, host.port, &host.username, password)
        .await?;
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

pub async fn list(host: &Host, password: &str, path: &str) -> Result<Vec<FileEntry>, String> {
    let sftp = open_sftp(host, password).await?;
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
    host: &Host,
    password: &str,
    local: &str,
    remote: &str,
) -> Result<(), String> {
    let data = tokio::fs::read(local)
        .await
        .map_err(|e| format!("read local file failed: {e}"))?;
    let sftp = open_sftp(host, password).await?;
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
    host: &Host,
    password: &str,
    remote: &str,
    local: &str,
) -> Result<(), String> {
    let sftp = open_sftp(host, password).await?;
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

pub async fn delete(host: &Host, password: &str, path: &str) -> Result<(), String> {
    let sftp = open_sftp(host, password).await?;
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
    host: &Host,
    password: &str,
    old_path: &str,
    new_path: &str,
) -> Result<(), String> {
    let sftp = open_sftp(host, password).await?;
    sftp.rename(old_path, new_path)
        .await
        .map_err(|e| format!("rename failed: {e}"))
}

pub async fn read_text(host: &Host, password: &str, path: &str) -> Result<String, String> {
    let sftp = open_sftp(host, password).await?;
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
    host: &Host,
    password: &str,
    path: &str,
    content: &str,
) -> Result<(), String> {
    let sftp = open_sftp(host, password).await?;
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
    use crate::models::AuthType;

    fn test_host() -> Host {
        Host::new(
            "SFTP CI".to_string(),
            std::env::var("TX_TEST_HOST").unwrap_or_else(|_| "47.100.33.169".to_string()),
            std::env::var("TX_TEST_PORT")
                .unwrap_or_else(|_| "22".to_string())
                .parse()
                .unwrap_or(22),
            std::env::var("TX_TEST_USER").unwrap_or_else(|_| "root".to_string()),
            AuthType::Password,
            "ci".to_string(),
            "默认".to_string(),
            vec![],
            None,
            None,
        )
    }

    fn test_password() -> Option<String> {
        std::env::var("TX_TEST_PASSWORD").ok()
    }

    #[tokio::test]
    async fn test_sftp_list_root() {
        let Some(pw) = test_password() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let host = test_host();
        let entries = list(&host, &pw, "/root").await.expect("list failed");
        assert!(!entries.is_empty(), "should list some entries in /root");
        assert!(entries.iter().any(|e| e.is_dir), "should contain dirs");
        eprintln!(
            "first entries: {:?}",
            entries.iter().take(5).map(|e| (&e.name, e.is_dir)).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_sftp_upload_download_roundtrip() {
        let Some(pw) = test_password() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let host = test_host();
        let dir = std::env::temp_dir();
        let remote = format!("/tmp/tianxuan_sftp_test_{}.txt", uuid::Uuid::new_v4());
        let local = dir.join("tianxuan_sftp_test.txt");

        // upload
        write_text(&host, &pw, &remote, "hello sftp roundtrip").await.expect("write_text");
        // read back
        let content = read_text(&host, &pw, &remote).await.expect("read_text");
        assert_eq!(content, "hello sftp roundtrip");
        // download to local
        download(&host, &pw, &remote, local.to_str().unwrap())
            .await
            .expect("download");
        let local_content = tokio::fs::read_to_string(&local).await.unwrap();
        assert_eq!(local_content, "hello sftp roundtrip");
        // upload local back to another remote path
        let remote2 = format!("{remote}.2");
        upload(&host, &pw, local.to_str().unwrap(), &remote2)
            .await
            .expect("upload");
        assert_eq!(read_text(&host, &pw, &remote2).await.unwrap(), "hello sftp roundtrip");
        // cleanup
        delete(&host, &pw, &remote).await.expect("delete remote");
        delete(&host, &pw, &remote2).await.expect("delete remote2");
        let _ = tokio::fs::remove_file(&local).await;
    }

    #[tokio::test]
    async fn test_sftp_rename() {
        let Some(pw) = test_password() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let host = test_host();
        let a = format!("/tmp/tianxuan_rename_{}.txt", uuid::Uuid::new_v4());
        let b = format!("{a}.renamed");
        write_text(&host, &pw, &a, "rename me").await.expect("write");
        rename(&host, &pw, &a, &b).await.expect("rename");
        let content = read_text(&host, &pw, &b).await.expect("read renamed");
        assert_eq!(content, "rename me");
        delete(&host, &pw, &b).await.expect("cleanup");
    }
}
