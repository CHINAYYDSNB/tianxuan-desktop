use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::services::host_service;
use crate::AppState;

#[tauri::command]
pub async fn open_panel_window(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<String, String> {
    let host = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        host_service::get(&conn, &id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "host not found".to_string())?
    };
    let panel_url = host
        .panel_url
        .clone()
        .ok_or_else(|| "此主机未配置面板地址".to_string())?;

    let label = format!("panel-{}", host.id);
    let existing = app.get_webview_window(&label);
    if let Some(win) = existing {
        let _ = win.set_focus();
        return Ok(label);
    }

    let parsed_url = panel_url
        .parse::<tauri::Url>()
        .map_err(|e| format!("invalid panel URL: {e}"))?;

    let win = WebviewWindowBuilder::new(
        &app,
        label.clone(),
        WebviewUrl::External(parsed_url),
    )
    .title(format!("{} 面板", host.name))
    .inner_size(1280.0, 820.0)
    .build()
    .map_err(|e| format!("open panel window failed: {e}"))?;

    win.show().map_err(|e| e.to_string())?;
    Ok(label)
}
