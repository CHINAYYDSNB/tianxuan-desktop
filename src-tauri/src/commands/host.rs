use tauri::State;

use crate::models::Host;
use crate::services::{host_service, keyring_store, metrics, ssh_client};
use crate::AppState;

fn keyring_key(host_id: &str) -> String {
    format!("host-password:{host_id}")
}

fn resolve_password(host: &Host) -> Result<String, String> {
    match &host.auth_type {
        crate::models::AuthType::Password => {
            keyring_store::get_password(&keyring_key(&host.id))?
                .ok_or_else(|| "password not stored in keyring".to_string())
        }
        crate::models::AuthType::Key => Err("key-based auth not yet supported".to_string()),
    }
}

#[tauri::command]
pub fn add_host(
    state: State<'_, AppState>,
    host: Host,
    password: Option<String>,
) -> Result<Host, String> {
    if let Some(pw) = password {
        keyring_store::set_password(&keyring_key(&host.id), &pw)?;
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    host_service::insert(&conn, &host).map_err(|e| e.to_string())?;
    Ok(host)
}

#[tauri::command]
pub fn update_host(
    state: State<'_, AppState>,
    host: Host,
    password: Option<String>,
) -> Result<Host, String> {
    if let Some(pw) = password {
        keyring_store::set_password(&keyring_key(&host.id), &pw)?;
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    host_service::update(&conn, &host).map_err(|e| e.to_string())?;
    Ok(host)
}

#[tauri::command]
pub fn delete_host(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let _ = keyring_store::delete_password(&keyring_key(&id));
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    host_service::delete(&conn, &id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_hosts(state: State<'_, AppState>) -> Result<Vec<Host>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    host_service::list(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_host(state: State<'_, AppState>, id: String) -> Result<Option<Host>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    host_service::get(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_connection(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let host = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        host_service::get(&conn, &id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "host not found".to_string())?
    };
    let password = resolve_password(&host)?;
    ssh_client::test_connection(&host, &password).await
}

#[tauri::command]
pub async fn collect_metrics(
    state: State<'_, AppState>,
    id: String,
) -> Result<metrics::HostMetrics, String> {
    let host = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        host_service::get(&conn, &id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "host not found".to_string())?
    };
    let password = match resolve_password(&host) {
        Ok(p) => p,
        Err(e) => return Ok(metrics::err_offline(&e)),
    };
    metrics::collect(&host, &password).await
}

#[tauri::command]
pub async fn exec_on_host(
    state: State<'_, AppState>,
    id: String,
    command: String,
) -> Result<serde_json::Value, String> {
    let host = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        host_service::get(&conn, &id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "host not found".to_string())?
    };
    let password = resolve_password(&host)?;
    let result = ssh_client::exec(&host, &password, &command).await?;
    Ok(serde_json::json!({
        "stdout": result.stdout,
        "stderr": result.stderr,
        "exit_code": result.exit_code,
    }))
}
