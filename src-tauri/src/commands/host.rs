use tauri::State;

use crate::models::Host;
use crate::services::host_service;
use crate::AppState;

#[tauri::command]
pub fn add_host(state: State<'_, AppState>, host: Host) -> Result<Host, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    host_service::insert(&conn, &host).map_err(|e| e.to_string())?;
    Ok(host)
}

#[tauri::command]
pub fn update_host(state: State<'_, AppState>, host: Host) -> Result<Host, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    host_service::update(&conn, &host).map_err(|e| e.to_string())?;
    Ok(host)
}

#[tauri::command]
pub fn delete_host(state: State<'_, AppState>, id: String) -> Result<(), String> {
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
