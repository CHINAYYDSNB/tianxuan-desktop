use std::sync::Arc;
use std::time::Duration;

use russh::client;
use russh::ChannelMsg;

use crate::models::Host;

pub struct SshResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<u32>,
}

struct SshHandler;

impl client::Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

fn build_config() -> Arc<client::Config> {
    Arc::new(client::Config {
        inactivity_timeout: Some(Duration::from_secs(10)),
        ..<_>::default()
    })
}

pub async fn connect_with_password(
    address: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<client::Handle<SshHandler>, String> {
    let config = build_config();
    let mut session = client::connect(config, (address, port), SshHandler {})
        .await
        .map_err(|e| format!("connect failed: {e}"))?;
    let auth_res = session
        .authenticate_password(username, password)
        .await
        .map_err(|e| format!("auth failed: {e}"))?;
    if !auth_res.success() {
        return Err("authentication rejected".to_string());
    }
    Ok(session)
}

pub async fn exec(
    host: &Host,
    password: &str,
    command: &str,
) -> Result<SshResult, String> {
    let mut session = connect_with_password(
        &host.address,
        host.port,
        &host.username,
        password,
    )
    .await?;
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

pub async fn test_connection(host: &Host, password: &str) -> Result<(), String> {
    let mut session = connect_with_password(
        &host.address,
        host.port,
        &host.username,
        password,
    )
    .await?;
    let result = run_command(&mut session, "echo tianxuan-pong").await?;
    if result.stdout.trim() != "tianxuan-pong" {
        return Err(format!("unexpected echo response: {:?}", result.stdout));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_host() -> Host {
        Host::new(
            "CI Test".to_string(),
            std::env::var("TX_TEST_HOST").unwrap_or_else(|_| "47.100.33.169".to_string()),
            std::env::var("TX_TEST_PORT")
                .unwrap_or_else(|_| "22".to_string())
                .parse()
                .unwrap_or(22),
            std::env::var("TX_TEST_USER").unwrap_or_else(|_| "root".to_string()),
            crate::models::AuthType::Password,
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
    async fn test_exec_uptime() {
        let Some(pw) = test_password() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let host = test_host();
        let result = exec(&host, &pw, "uptime").await.expect("exec failed");
        assert_eq!(result.exit_code, Some(0));
        assert!(!result.stdout.is_empty(), "uptime output empty");
        eprintln!("uptime: {}", result.stdout.trim());
    }

    #[tokio::test]
    async fn test_echo_roundtrip() {
        let Some(pw) = test_password() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let host = test_host();
        let result = exec(&host, &pw, "echo tianxuan-pong").await.expect("exec failed");
        assert_eq!(result.stdout.trim(), "tianxuan-pong");
    }

    #[tokio::test]
    async fn test_wrong_password() {
        let Some(pw) = test_password() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let host = test_host();
        let res = exec(&host, "definitely-wrong-password-123", "uptime").await;
        assert!(res.is_err(), "wrong password should fail");
        eprintln!("wrong password error: {:?}", res.err());
    }

    #[tokio::test]
    async fn test_exec_failure_exit_code() {
        let Some(pw) = test_password() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let host = test_host();
        let result = exec(&host, &pw, "exit 42").await.expect("exec failed");
        assert_eq!(result.exit_code, Some(42));
    }

    #[tokio::test]
    async fn test_connection_ok() {
        let Some(pw) = test_password() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let host = test_host();
        test_connection(&host, &pw).await.expect("connection should succeed");
    }
}
