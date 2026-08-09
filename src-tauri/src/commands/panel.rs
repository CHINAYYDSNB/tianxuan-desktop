use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::models::{NewPanel, Panel};
use crate::services::panel_service;
use crate::AppState;

#[tauri::command]
pub fn add_panel(state: State<'_, AppState>, panel: NewPanel) -> Result<Panel, String> {
    let panel = panel.into_panel();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    panel_service::insert(&conn, &panel).map_err(|e| e.to_string())?;
    Ok(panel)
}

#[tauri::command]
pub fn update_panel(state: State<'_, AppState>, panel: Panel) -> Result<Panel, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    panel_service::update(&conn, &panel).map_err(|e| e.to_string())?;
    Ok(panel)
}

#[tauri::command]
pub fn delete_panel(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    panel_service::delete(&conn, &id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_panels(state: State<'_, AppState>) -> Result<Vec<Panel>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    panel_service::list(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_panel(state: State<'_, AppState>, id: String) -> Result<Option<Panel>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    panel_service::get(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_panel_window(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<String, String> {
    let panel = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        panel_service::get(&conn, &id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "panel not found".to_string())?
    };

    let label = format!("panel-{}", panel.id);
    let existing = app.get_webview_window(&label);
    if let Some(win) = existing {
        let _ = win.set_focus();
        return Ok(label);
    }

    let parsed_url = panel
        .url
        .parse::<tauri::Url>()
        .map_err(|e| format!("invalid panel URL: {e}"))?;

    let win = WebviewWindowBuilder::new(
        &app,
        label.clone(),
        WebviewUrl::External(parsed_url),
    )
    .title(format!("{} 面板", panel.name))
    .inner_size(1280.0, 820.0)
    .build()
    .map_err(|e| format!("open panel window failed: {e}"))?;

    win.show().map_err(|e| e.to_string())?;
    Ok(label)
}
