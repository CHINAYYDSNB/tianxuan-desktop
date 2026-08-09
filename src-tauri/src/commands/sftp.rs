use tauri::State;

use crate::services::{
    host_service, keyring_store, sftp_client, ssh_client::SshConfig,
};
use crate::AppState;

fn keyring_key(host_id: &str) -> String {
    format!("host-password:{host_id}")
}

fn resolve_password(host: &crate::models::Host) -> Result<String, String> {
    match &host.auth_type {
        crate::models::AuthType::Password => {
            keyring_store::get_password(&keyring_key(&host.id))?
                .ok_or_else(|| "password not stored in keyring".to_string())
        }
        crate::models::AuthType::Key => Err("key-based auth not yet supported".to_string()),
    }
}

async fn get_host_config(state: &State<'_, AppState>, id: &str) -> Result<SshConfig, String> {
    let host = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        host_service::get(&conn, id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "host not found".to_string())?
    };
    let password = resolve_password(&host)?;
    Ok(SshConfig::from_host_password(&host, &password))
}

#[tauri::command]
pub async fn sftp_list(
    state: State<'_, AppState>,
    id: String,
    path: String,
) -> Result<Vec<sftp_client::FileEntry>, String> {
    let config = get_host_config(&state, &id).await?;
    sftp_client::list(&config, &path).await
}

#[tauri::command]
pub async fn sftp_upload(
    state: State<'_, AppState>,
    id: String,
    local: String,
    remote: String,
) -> Result<(), String> {
    let config = get_host_config(&state, &id).await?;
    sftp_client::upload(&config, &local, &remote).await
}

#[tauri::command]
pub async fn sftp_download(
    state: State<'_, AppState>,
    id: String,
    remote: String,
    local: String,
) -> Result<(), String> {
    let config = get_host_config(&state, &id).await?;
    sftp_client::download(&config, &remote, &local).await
}

#[tauri::command]
pub async fn sftp_delete(
    state: State<'_, AppState>,
    id: String,
    path: String,
) -> Result<(), String> {
    let config = get_host_config(&state, &id).await?;
    sftp_client::delete(&config, &path).await
}

#[tauri::command]
pub async fn sftp_rename(
    state: State<'_, AppState>,
    id: String,
    old_path: String,
    new_path: String,
) -> Result<(), String> {
    let config = get_host_config(&state, &id).await?;
    sftp_client::rename(&config, &old_path, &new_path).await
}

#[tauri::command]
pub async fn sftp_read_text(
    state: State<'_, AppState>,
    id: String,
    path: String,
) -> Result<String, String> {
    let config = get_host_config(&state, &id).await?;
    sftp_client::read_text(&config, &path).await
}

#[tauri::command]
pub async fn sftp_write_text(
    state: State<'_, AppState>,
    id: String,
    path: String,
    content: String,
) -> Result<(), String> {
    let config = get_host_config(&state, &id).await?;
    sftp_client::write_text(&config, &path, &content).await
}
