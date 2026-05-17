use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod config;
pub mod types;
pub mod commands;
mod scanner;

use types::{ScanStore, PauseStore, CancelStore};

#[tauri::command]
fn select_directory() -> Result<String, String> {
    Ok(String::new())
}

#[tauri::command]
fn get_config(app_handle: tauri::AppHandle) -> config::AppConfig {
    config::AppConfig::load(&app_handle)
}

#[tauri::command]
fn save_config(app_handle: tauri::AppHandle, cfg: config::AppConfig) -> Result<(), String> {
    cfg.save(&app_handle)
}

#[tauri::command]
fn reset_config(app_handle: tauri::AppHandle) -> config::AppConfig {
    let cfg = config::AppConfig::default();
    let _ = cfg.save(&app_handle);
    cfg
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let scan_store: ScanStore = Arc::new(Mutex::new(HashMap::new()));
    let pause_store = PauseStore(Arc::new(Mutex::new(HashMap::new())));
    let cancel_store = CancelStore(Arc::new(Mutex::new(HashMap::new())));
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(scan_store)
        .manage(pause_store)
        .manage(cancel_store)
        .invoke_handler(tauri::generate_handler![
            select_directory,
            get_config,
            save_config,
            reset_config,
            commands::scan::start_scan,
            commands::scan::scan_sub_directory,
            commands::scan::get_scan_progress,
            commands::scan::cancel_scan,
            commands::scan::pause_scan,
            commands::scan::resume_scan,
            commands::scan::respond_confirmation,
            commands::file_ops::read_file_preview,
            commands::file_ops::read_image_as_base64,
            commands::file_ops::move_to_trash,
            commands::file_ops::reveal_in_finder,
            commands::file_ops::get_file_info,
            commands::system::play_system_sound,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
