use tauri::State;

use crate::models::AuthType;
use crate::services::{batch, host_service, keyring_store};
use crate::AppState;

fn keyring_key(host_id: &str) -> String {
    format!("host-password:{host_id}")
}

#[tauri::command]
pub async fn batch_exec(
    state: State<'_, AppState>,
    host_ids: Vec<String>,
    cmd: String,
) -> Result<Vec<batch::BatchResult>, String> {
    let hosts = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let mut found = Vec::new();
        for id in &host_ids {
            if let Some(h) = host_service::get(&conn, id).map_err(|e| e.to_string())? {
                found.push(h);
            }
        }
        found
    };

    let mut passwords = Vec::new();
    for host in &hosts {
        let password = match &host.auth_type {
            AuthType::Password => {
                keyring_store::get_password(&keyring_key(&host.id))?
                    .ok_or_else(|| format!("password not stored for {}", host.name))?
            }
            AuthType::Key => return Err("key-based auth not yet supported".to_string()),
        };
        passwords.push(password);
    }

    let results = batch::execute(hosts, passwords, &cmd).await;

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let success = results.iter().filter(|r| r.success).count();
        let fail = results.len() - success;
        let _ = batch::save_history(&conn, &cmd, results.len(), success, fail);
    }

    Ok(results)
}

#[tauri::command]
pub fn list_command_history(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    batch::list_history(&conn).map_err(|e| e.to_string())
}
