use tauri::{Manager, State};

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
pub async fn open_panel_tab(
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
    let window = app
        .get_window("main")
        .ok_or_else(|| "main window not found".to_string())?;
    let tabs = state.panel_tabs.clone();
    // create the child webview on a dedicated thread to avoid Windows deadlocks
    tauri::async_runtime::spawn_blocking(move || tabs.open(&window, &panel))
        .await
        .map_err(|e| format!("spawn failed: {e}"))?
}

#[tauri::command]
pub fn switch_panel_tab(
    state: State<'_, AppState>,
    label: String,
) -> Result<(), String> {
    state.panel_tabs.switch(&label)
}

#[tauri::command]
pub fn hide_panel_tabs(state: State<'_, AppState>) -> Result<(), String> {
    state.panel_tabs.hide_all();
    Ok(())
}

#[tauri::command]
pub fn close_panel_tab(
    state: State<'_, AppState>,
    label: String,
) -> Result<(), String> {
    state.panel_tabs.close(&label)
}

#[tauri::command]
pub fn list_panel_tabs(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.panel_tabs.open_labels())
}

#[tauri::command]
pub fn active_panel_tab(state: State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(state.panel_tabs.active_label())
}
