use rusqlite::{params, Connection};
use serde::Serialize;

use crate::models::Host;
use crate::services::ssh_client;

#[derive(Debug, Clone, Serialize)]
pub struct BatchResult {
    pub host_id: String,
    pub host_name: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<u32>,
    pub success: bool,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

pub async fn execute(
    hosts: Vec<Host>,
    passwords: Vec<String>,
    command: &str,
) -> Vec<BatchResult> {
    let mut tasks = Vec::new();
    for (host, password) in hosts.into_iter().zip(passwords.into_iter()) {
        let command = command.to_string();
        tasks.push(tokio::spawn(async move {
            let start = std::time::Instant::now();
            match ssh_client::exec(&host, &password, &command).await {
                Ok(result) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    BatchResult {
                        host_id: host.id.clone(),
                        host_name: host.name.clone(),
                        stdout: result.stdout,
                        stderr: result.stderr,
                        exit_code: result.exit_code,
                        success: result.exit_code == Some(0),
                        elapsed_ms: elapsed,
                        error: None,
                    }
                }
                Err(e) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    BatchResult {
                        host_id: host.id.clone(),
                        host_name: host.name.clone(),
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: None,
                        success: false,
                        elapsed_ms: elapsed,
                        error: Some(e),
                    }
                }
            }
        }));
    }

    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(r) => results.push(r),
            Err(e) => results.push(BatchResult {
                host_id: String::new(),
                host_name: "任务崩溃".to_string(),
                stdout: String::new(),
                stderr: String::new(),
                exit_code: None,
                success: false,
                elapsed_ms: 0,
                error: Some(e.to_string()),
            }),
        }
    }
    results
}

pub fn save_history(
    conn: &Connection,
    command: &str,
    host_count: usize,
    success_count: usize,
    fail_count: usize,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO command_history (id, command, host_count, executed_at, success_count, fail_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            uuid::Uuid::new_v4().to_string(),
            command,
            host_count as i64,
            chrono::Utc::now().to_rfc3339(),
            success_count as i64,
            fail_count as i64,
        ],
    )?;
    Ok(())
}

pub fn list_history(conn: &Connection) -> rusqlite::Result<Vec<serde_json::Value>> {
    let mut stmt = conn.prepare(
        "SELECT id, command, host_count, executed_at, success_count, fail_count
         FROM command_history ORDER BY executed_at DESC LIMIT 100",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "command": row.get::<_, String>(1)?,
            "host_count": row.get::<_, i64>(2)?,
            "executed_at": row.get::<_, String>(3)?,
            "success_count": row.get::<_, i64>(4)?,
            "fail_count": row.get::<_, i64>(5)?,
        }))
    })?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AuthType;

    fn test_host(name: &str) -> Host {
        Host::new(
            name.to_string(),
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

    #[tokio::test]
    async fn test_batch_concurrent_exec() {
        let Some(pw) = std::env::var("TX_TEST_PASSWORD").ok() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        // simulate two hosts (same server) to verify concurrency + aggregation
        let hosts = vec![test_host("node-a"), test_host("node-b")];
        let passwords = vec![pw.clone(), pw];
        let results = execute(hosts, passwords, "hostname && echo done").await;

        assert_eq!(results.len(), 2);
        for r in &results {
            assert!(r.success, "both should succeed: {:?}", r.error);
            assert!(!r.stdout.is_empty());
            assert!(r.elapsed_ms < 10000);
        }
        eprintln!(
            "batch results: {} / {} succeeded, elapsed {:?}ms",
            results.iter().filter(|r| r.success).count(),
            results.len(),
            results.iter().map(|r| r.elapsed_ms).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_batch_failure_aggregated() {
        let Some(pw) = std::env::var("TX_TEST_PASSWORD").ok() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let hosts = vec![test_host("node-a"), test_host("node-b")];
        let passwords = vec![pw.clone(), pw];
        let results = execute(hosts, passwords, "exit 3").await;

        assert_eq!(results.len(), 2);
        assert!(
            results.iter().all(|r| !r.success),
            "exit 3 should be marked as failure"
        );
        assert!(results.iter().all(|r| r.exit_code == Some(3)));
    }

    #[test]
    fn test_save_and_list_history() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrate::run(&conn).unwrap();
        save_history(&conn, "uptime", 2, 1, 1).unwrap();
        save_history(&conn, "df -h", 1, 1, 0).unwrap();

        let history = list_history(&conn).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0]["command"], "df -h");
        assert_eq!(history[0]["success_count"], 1);
        assert_eq!(history[0]["fail_count"], 0);
    }
}
