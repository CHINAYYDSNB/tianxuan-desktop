use tauri::{Emitter, Manager, State};

use crate::services::{host_service, keyring_store};
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

#[tauri::command]
pub async fn ssh_open_session(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
    session_id: String,
    window: tauri::WebviewWindow,
) -> Result<(), String> {
    let host = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        host_service::get(&conn, &id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "host not found".to_string())?
    };
    let password = resolve_password(&host)?;
    let session_id = if session_id.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        session_id
    };

    let mut rx = state.terminal.open(&host, &password, session_id.clone()).await?;

    // forward output to the frontend via Tauri event
    let win_label = window.label().to_string();
    let sid = session_id.clone();
    tokio::spawn(async move {
        while let Some(chunk) = rx.recv().await {
            let payload = serde_json::json!({
                "session_id": sid,
                "data": chunk,
            });
            if let Some(win) = app.clone().get_webview_window(&win_label) {
                let _ = win.emit("terminal-output", payload);
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn ssh_write(
    state: State<'_, AppState>,
    session_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    state.terminal.write(&session_id, data).await
}

#[tauri::command]
pub async fn ssh_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u32,
    rows: u32,
) -> Result<(), String> {
    state.terminal.resize(&session_id, cols, rows).await
}

#[tauri::command]
pub async fn ssh_close_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    state.terminal.close(&session_id).await
}
