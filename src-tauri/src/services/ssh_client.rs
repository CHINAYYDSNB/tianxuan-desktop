use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use russh::client;
use russh::ChannelMsg;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::models::Host;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SshAuth {
    Password { password: String },
    Key { path: String, passphrase: Option<String> },
}

#[derive(Debug, Clone)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: SshAuth,
}

impl SshConfig {
    pub fn from_host_password(host: &Host, password: &str) -> Self {
        Self {
            host: host.address.clone(),
            port: host.port,
            username: host.username.clone(),
            auth: SshAuth::Password {
                password: password.to_string(),
            },
        }
    }
}

pub struct SshResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<u32>,
}

fn known_hosts_path() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".ssh").join("known_hosts")
}

pub struct SshHandler {
    host: String,
    port: u16,
    pub disconnect_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
}

impl SshHandler {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            disconnect_tx: None,
        }
    }

    pub fn with_disconnect_sender(
        mut self,
        tx: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> Self {
        self.disconnect_tx = Some(tx);
        self
    }

    fn host_identifier(&self) -> String {
        if self.port != 22 {
            format!("[{}]:{}", self.host, self.port)
        } else {
            self.host.clone()
        }
    }

    /// Append host key to known_hosts (TOFU), creating the file if needed.
    fn append_known_host(&self, host_entry: &str) {
        let path = known_hosts_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(f, "{host_entry}");
        }
    }
}

impl client::Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        use russh::keys::PublicKeyBase64;
        let key_type = server_public_key.algorithm().to_string();
        let key_base64 = server_public_key.public_key_base64();
        let host_identifier = self.host_identifier();
        let host_entry = format!("{host_identifier} {key_type} {key_base64}");

        let Ok(content) = std::fs::read_to_string(known_hosts_path()) else {
            // no known_hosts file yet -> TOFU accept
            self.append_known_host(&host_entry);
            return Ok(true);
        };

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[0] == host_identifier {
                if parts[1] == key_type && parts[2] == key_base64 {
                    return Ok(true);
                }
                // host seen with different key -> reject (host key changed)
                return Ok(false);
            }
        }

        // unknown host -> TOFU accept
        self.append_known_host(&host_entry);
        Ok(true)
    }

    async fn disconnected(
        &mut self,
        reason: client::DisconnectReason<Self::Error>,
    ) -> Result<(), Self::Error> {
        let message = match reason {
            client::DisconnectReason::ReceivedDisconnect(info) => {
                format!("SSH server disconnected: {}", info.message)
            }
            client::DisconnectReason::Error(error) => {
                format!("SSH connection error: {error}")
            }
        };
        if let Some(tx) = &self.disconnect_tx {
            let _ = tx.send(message);
        }
        Ok(())
    }
}

pub type SshRawHandle = Arc<Mutex<client::Handle<SshHandler>>>;

fn build_config() -> Arc<client::Config> {
    Arc::new(client::Config {
        inactivity_timeout: Some(Duration::from_secs(60)),
        keepalive_interval: Some(Duration::from_secs(30)),
        ..<_>::default()
    })
}

pub async fn connect(config: &SshConfig) -> Result<SshRawHandle, String> {
    let ssh_config = build_config();
    let mut session = client::connect(
        ssh_config,
        (config.host.as_str(), config.port),
        SshHandler::new(config.host.clone(), config.port),
    )
    .await
    .map_err(|e| format!("connect failed: {e}"))?;

    let auth_res = match &config.auth {
        SshAuth::Password { password } => session
            .authenticate_password(&config.username, password)
            .await
            .map_err(|e| format!("auth failed: {e}"))?,
        SshAuth::Key { path, passphrase } => {
            let key = load_secret_key(Path::new(path), passphrase.as_deref())?;
            let hash = session.best_supported_rsa_hash().await.map_err(|e| e.to_string())?;
            session
                .authenticate_publickey(
                    &config.username,
                    russh::keys::PrivateKeyWithHashAlg::new(Arc::new(key), hash.flatten()),
                )
                .await
                .map_err(|e| format!("key auth failed: {e}"))?
        }
    };

    if !auth_res.success() {
        return Err("authentication rejected".to_string());
    }
    Ok(Arc::new(Mutex::new(session)))
}

fn load_secret_key(
    path: &Path,
    passphrase: Option<&str>,
) -> Result<ssh_key::private::PrivateKey, String> {
    russh::keys::load_secret_key(path, passphrase).map_err(|e| format!("load key failed: {e}"))
}

pub async fn exec(
    handle: &SshRawHandle,
    command: &str,
) -> Result<SshResult, String> {
    let mut session = handle.lock().await;
    run_command(&mut session, command).await
}

pub async fn run_command(
    session: &mut client::Handle<SshHandler>,
    command: &str,
) -> Result<SshResult, String> {
    let mut channel = session
        .channel_open_session()
        .await
        .map_err(|e| format!("open channel failed: {e}"))?;
    channel
        .exec(true, command)
        .await
        .map_err(|e| format!("exec failed: {e}"))?;

    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut exit_code = None;

    loop {
        let Some(msg) = channel.wait().await else {
            break;
        };
        match msg {
            ChannelMsg::Data { ref data } => {
                stdout.push_str(&String::from_utf8_lossy(data));
            }
            ChannelMsg::ExtendedData { ref data, .. } => {
                stderr.push_str(&String::from_utf8_lossy(data));
            }
            ChannelMsg::ExitStatus { exit_status } => {
                exit_code = Some(exit_status);
            }
            _ => {}
        }
    }

    Ok(SshResult {
        stdout,
        stderr,
        exit_code,
    })
}

pub async fn test_connection(config: &SshConfig) -> Result<(), String> {
    let handle = connect(config).await?;
    let result = exec(&handle, "echo tianxuan-pong").await?;
    if result.stdout.trim() != "tianxuan-pong" {
        return Err(format!("unexpected echo response: {:?}", result.stdout));
    }
    Ok(())
}
