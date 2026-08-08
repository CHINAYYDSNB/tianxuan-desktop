use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

pub mod commands;
pub mod db;
pub mod models;
pub mod services;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub terminal: services::terminal::SessionManager,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg(test)]
mod tests {
    use super::greet;

    #[test]
    fn test_greet() {
        assert_eq!(
            greet("tianxuan"),
            "Hello, tianxuan! You've been greeted from Rust!"
        );
    }
}

fn init_db(app: &tauri::App) -> Result<Connection, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?;
    std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
    let db_path = app_data_dir.join("tianxuan.db");
    db::open(&db_path).map_err(|e| format!("failed to open database: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let conn = init_db(app)?;
            app.manage(AppState {
                db: Mutex::new(conn),
                terminal: services::terminal::SessionManager::new(),
            });
            Ok(())
        })
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::host::add_host,
            commands::host::update_host,
            commands::host::delete_host,
            commands::host::list_hosts,
            commands::host::get_host,
            commands::host::test_connection,
            commands::host::exec_on_host,
            commands::host::collect_metrics,
            commands::batch::batch_exec,
            commands::batch::list_command_history,
            commands::panel::open_panel_window,
            commands::ssh::ssh_open_session,
            commands::ssh::ssh_write,
            commands::ssh::ssh_resize,
            commands::ssh::ssh_close_session,
            commands::sftp::sftp_list,
            commands::sftp::sftp_upload,
            commands::sftp::sftp_download,
            commands::sftp::sftp_delete,
            commands::sftp::sftp_rename,
            commands::sftp::sftp_read_text,
            commands::sftp::sftp_write_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
