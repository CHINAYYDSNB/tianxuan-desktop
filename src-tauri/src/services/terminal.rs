use std::collections::HashMap;
use std::sync::Arc;

use russh::ChannelMsg;
use tokio::sync::{mpsc, Mutex};

use crate::services::ssh_client::{SshRawHandle, SshResult};

const OUTPUT_BUF: usize = 65536;
const INPUT_BUF: usize = 65536;

pub enum SessionCommand {
    Write(Vec<u8>),
    Resize(u32, u32),
    Close,
}

pub struct ActiveSession {
    pub command_tx: mpsc::Sender<SessionCommand>,
}

pub type SessionHandle = Arc<Mutex<HashMap<String, ActiveSession>>>;

pub struct SessionManager {
    sessions: SessionHandle,
    connections: Arc<Mutex<HashMap<String, SshRawHandle>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Store an authenticated SSH connection keyed by host identity for reuse.
    pub async fn store_connection(&self, key: &str, handle: SshRawHandle) {
        self.connections.lock().await.insert(key.to_string(), handle);
    }

    /// Open a PTY shell channel on an existing authenticated connection.
    pub async fn open(
        &self,
        handle: &SshRawHandle,
        session_id: String,
    ) -> Result<mpsc::Receiver<String>, String> {
        let session = handle.lock().await;

        let mut channel = session
            .channel_open_session()
            .await
            .map_err(|e| format!("open channel failed: {e}"))?;

        channel
            .request_pty(false, "xterm-256color", 80, 24, 0, 0, &[])
            .await
            .map_err(|e| format!("request_pty failed: {e}"))?;

        channel
            .request_shell(true)
            .await
            .map_err(|e| format!("shell failed: {e}"))?;

        let (cmd_tx, mut cmd_rx) = mpsc::channel::<SessionCommand>(INPUT_BUF);
        let (out_tx, out_rx) = mpsc::channel::<String>(OUTPUT_BUF);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(SessionCommand::Write(data)) => {
                                let bytes: bytes::Bytes = data.into();
                                if channel.data_bytes(bytes).await.is_err() {
                                    break;
                                }
                            }
                            Some(SessionCommand::Resize(w, h)) => {
                                let _ = channel.window_change(w, h, 0, 0).await;
                            }
                            Some(SessionCommand::Close) | None => {
                                let _ = channel.close().await;
                                break;
                            }
                        }
                    }
                    msg = channel.wait() => {
                        let Some(msg) = msg else {
                            break;
                        };
                        match msg {
                            ChannelMsg::Data { ref data } => {
                                if out_tx.send(String::from_utf8_lossy(data).to_string()).await.is_err() {
                                    break;
                                }
                            }
                            ChannelMsg::ExtendedData { ref data, .. } => {
                                if out_tx.send(String::from_utf8_lossy(data).to_string()).await.is_err() {
                                    break;
                                }
                            }
                            ChannelMsg::ExitStatus { .. } => {
                                if out_tx.send("\u{0}[EXIT]".to_string()).await.is_err() {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        let mut map = self.sessions.lock().await;
        map.insert(session_id, ActiveSession { command_tx: cmd_tx });
        drop(map);

        Ok(out_rx)
    }

    pub async fn write(&self, session_id: &str, data: Vec<u8>) -> Result<(), String> {
        let tx = {
            let map = self.sessions.lock().await;
            let active = map
                .get(session_id)
                .ok_or_else(|| "session not found".to_string())?;
            active.command_tx.clone()
        };
        tx.send(SessionCommand::Write(data))
            .await
            .map_err(|e| format!("write failed: {e}"))
    }

    pub async fn resize(&self, session_id: &str, cols: u32, rows: u32) -> Result<(), String> {
        let tx = {
            let map = self.sessions.lock().await;
            let active = map
                .get(session_id)
                .ok_or_else(|| "session not found".to_string())?;
            active.command_tx.clone()
        };
        tx.send(SessionCommand::Resize(cols, rows))
            .await
            .map_err(|e| format!("resize failed: {e}"))
    }

    pub async fn close(&self, session_id: &str) -> Result<(), String> {
        let active = {
            let mut map = self.sessions.lock().await;
            map.remove(session_id)
        };
        if let Some(active) = active {
            let _ = active.command_tx.send(SessionCommand::Close).await;
        }
        Ok(())
    }
}

pub async fn exec_once(handle: &SshRawHandle, command: &str) -> Result<SshResult, String> {
    crate::services::ssh_client::exec(handle, command).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AuthType, Host};
    use crate::services::ssh_client::{connect, SshAuth, SshConfig};
    use std::time::Duration;

    fn test_host() -> Host {
        Host::new(
            "Term Test".to_string(),
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
        )
    }

    fn test_config() -> Option<SshConfig> {
        let pw = std::env::var("TX_TEST_PASSWORD").ok()?;
        let h = test_host();
        Some(SshConfig {
            host: h.address,
            port: h.port,
            username: h.username,
            auth: SshAuth::Password { password: pw },
        })
    }

    #[tokio::test]
    async fn test_terminal_echo_roundtrip() {
        let Some(config) = test_config() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let handle = connect(&config).await.expect("connect");
        let mgr = SessionManager::new();
        let mut rx = mgr.open(&handle, "t1".to_string()).await.expect("open session");

        tokio::time::sleep(Duration::from_millis(500)).await;

        while let Ok(Some(_)) = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
            // drain initial output
        }

        mgr.write("t1", b"echo tianxuan-term-test\r".to_vec())
            .await
            .expect("write");

        let mut collected = String::new();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
                Ok(Some(chunk)) => {
                    collected.push_str(&chunk);
                    if collected.contains("tianxuan-term-test") {
                        break;
                    }
                }
                _ => {
                    if collected.is_empty() {
                        continue;
                    }
                    break;
                }
            }
        }

        mgr.close("t1").await.expect("close");
        assert!(
            collected.contains("tianxuan-term-test"),
            "expected echo in output, got: {collected:?}"
        );
    }

    #[tokio::test]
    async fn test_reuse_connection_for_exec() {
        let Some(config) = test_config() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let handle = connect(&config).await.expect("connect");
        // two execs on the same connection
        let r1 = exec_once(&handle, "echo one").await.expect("exec1");
        let r2 = exec_once(&handle, "echo two").await.expect("exec2");
        assert_eq!(r1.stdout.trim(), "one");
        assert_eq!(r2.stdout.trim(), "two");
    }
}
