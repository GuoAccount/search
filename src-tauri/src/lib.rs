use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod config;
pub mod types;
pub mod commands;
mod scanner;
mod ocr;

use types::{ScanStore, CancelStore, ChannelStore};

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

#[tauri::command]
fn get_log_content(app_handle: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;
    use std::io::{BufRead, BufReader, Seek, SeekFrom};
    let log_dir = app_handle.path().app_log_dir().map_err(|e| e.to_string())?;
    let log_file = log_dir.join("lumina.log");
    
    if !log_file.exists() {
        return Ok(String::new());
    }

    let file = std::fs::File::open(&log_file).map_err(|e| e.to_string())?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    let file_size = metadata.len();

    // Read last ~100KB max, then take last 500 lines
    let read_size = file_size.min(100_000);
    let mut reader = BufReader::new(file);
    let _ = reader.seek(SeekFrom::End(-(read_size as i64)));
    
    let mut lines: Vec<String> = reader
        .lines()
        .filter_map(|l| l.ok())
        .collect();
    
    // Drop the first (potentially partial) line
    if lines.len() > 1 {
        lines.remove(0);
    }
    
    // Keep last 500 lines
    let total = lines.len();
    if total > 500 {
        lines = lines.split_off(total - 500);
    }
    
    Ok(lines.join("\n"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let scan_store: ScanStore = Arc::new(Mutex::new(HashMap::new()));
    let cancel_store = CancelStore(Arc::new(Mutex::new(HashMap::new())));
    let channel_store: ChannelStore = Arc::new(Mutex::new(HashMap::new()));
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("lumina".into()),
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                ])
                .level(log::LevelFilter::Warn)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(3))
                .max_file_size(5_000_000)
                .build(),
        )
        .manage(scan_store)
        .manage(cancel_store)
        .manage(channel_store)
        .invoke_handler(tauri::generate_handler![
            select_directory,
            get_config,
            save_config,
            reset_config,
            get_log_content,
            commands::scan::start_scan,
            commands::scan::get_scan_progress,
            commands::scan::cancel_scan,
            commands::scan::respond_confirmation,
            commands::file_ops::read_file_preview,
            commands::file_ops::read_image_as_base64,
            commands::file_ops::move_to_trash,
            commands::file_ops::reveal_in_finder,
            commands::file_ops::get_file_info,
            commands::system::play_system_sound,
            commands::system::open_config_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
