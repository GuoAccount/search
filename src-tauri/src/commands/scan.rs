use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::path::PathBuf;
use tauri::Emitter;
use uuid::Uuid;

use crate::config;
use crate::scanner::{ScanCallback, ScanResult};
use crate::types::{PendingConfirmation, DirWork, ScanConfig, ScanProgress, SkippedDir, CancelStore, ScanStore, ChannelStore};

#[tauri::command]
pub async fn start_scan(
    app_handle: tauri::AppHandle,
    config: ScanConfig,
    state: tauri::State<'_, ScanStore>,
    cancel_state: tauri::State<'_, CancelStore>,
    channel_state: tauri::State<'_, ChannelStore>,
) -> Result<String, String> {
    let scan_id = Uuid::new_v4().to_string();
    log::info!("Starting scan with id={}, keyword='{}', path='{}'", 
        scan_id, config.keyword, config.path);

    let progress = ScanProgress {
        scan_id: scan_id.clone(),
        parent_scan_id: None,
        status: "scanning".to_string(),
        files_scanned: 0,
        results_found: 0,
        current_path: String::new(),
        results: Vec::new(),
        pending_confirmations: Vec::new(),
        skipped_dirs: Vec::new(),
    };

    state.lock().unwrap().insert(scan_id.clone(), progress);

    let should_cancel = Arc::new(Mutex::new(false));

    cancel_state.0.lock().unwrap().insert(scan_id.clone(), should_cancel.clone());

    // Create work channel
    let (work_tx, work_rx) = mpsc::channel::<DirWork>();

    // Store work_tx for respond_confirmation to use
    channel_state.lock().unwrap().insert(scan_id.clone(), work_tx.clone());

    let app_config = config::AppConfig::load(&app_handle);

    let store = state.inner().clone();
    let channel_store = channel_state.inner().clone();
    let sid = scan_id.clone();

    tokio::spawn(async move {
        let store_for_result = store.clone();
        let store_for_progress = store.clone();
        let store_for_confirmation = store.clone();
        let store_for_skipped = store.clone();
        let sid_for_result = sid.clone();
        let sid_for_progress = sid.clone();
        let sid_for_confirmation = sid.clone();
        let sid_for_skipped = sid.clone();
        let app_handle_clone = app_handle.clone();

        let callback = ScanCallback {
            on_result: Box::new(move |result: ScanResult| {
                let mut store_guard = store_for_result.lock().unwrap();
                if let Some(progress) = store_guard.get_mut(&sid_for_result) {
                    progress.results.push(result);
                    progress.results_found = progress.results.len() as u32;
                }
            }),
            on_progress: Box::new(move |files_scanned: u32, current_path: String| {
                let mut store_guard = store_for_progress.lock().unwrap();
                if let Some(progress) = store_guard.get_mut(&sid_for_progress) {
                    progress.files_scanned = files_scanned;
                    progress.current_path = current_path;
                }
            }),
            on_confirmation_needed: Box::new(move |confirmation: PendingConfirmation| {
                let mut store_guard = store_for_confirmation.lock().unwrap();
                if let Some(progress) = store_guard.get_mut(&sid_for_confirmation) {
                    progress.pending_confirmations.push(confirmation.clone());
                }
                let _ = app_handle_clone.emit("confirmation-needed", confirmation);
            }),
            on_dir_skipped: Box::new(move |skipped: SkippedDir| {
                let mut store_guard = store_for_skipped.lock().unwrap();
                if let Some(progress) = store_guard.get_mut(&sid_for_skipped) {
                    progress.skipped_dirs.push(skipped);
                }
            }),
            should_cancel: should_cancel.clone(),
        };

        crate::scanner::scan_directory(config, app_config, callback, work_tx, work_rx);

        // Cleanup: remove channel from store (this drops the last work_tx clone)
        channel_store.lock().unwrap().remove(&sid);

        let mut store_guard = store.lock().unwrap();
        if let Some(progress) = store_guard.get_mut(&sid) {
            progress.status = "completed".to_string();
            progress.current_path = String::new();
            log::info!("Scan {} completed: {} results found, {} files scanned", 
                sid, progress.results_found, progress.files_scanned);
        }
    });

    Ok(scan_id)
}

#[tauri::command]
pub fn get_scan_progress(
    scan_id: String,
    state: tauri::State<'_, ScanStore>,
) -> Result<ScanProgress, String> {
    let store = state.lock().unwrap();
    store.get(&scan_id).cloned().ok_or_else(|| "Scan not found".to_string())
}

#[tauri::command]
pub fn cancel_scan(
    scan_id: String,
    state: tauri::State<'_, ScanStore>,
    cancel_state: tauri::State<'_, CancelStore>,
) -> Result<(), String> {
    log::info!("Cancelling scan: {}", scan_id);
    
    let mut store = state.lock().unwrap();
    if let Some(progress) = store.get_mut(&scan_id) {
        progress.status = "cancelled".to_string();
    }

    let cancel_store = cancel_state.0.lock().unwrap();
    if let Some(cancel_flag) = cancel_store.get(&scan_id) {
        *cancel_flag.lock().unwrap() = true;
    }

    Ok(())
}

#[tauri::command]
pub fn respond_confirmation(
    scan_id: String,
    confirmation_id: String,
    allow: bool,
    remember: bool,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, ScanStore>,
    channel_state: tauri::State<'_, ChannelStore>,
) -> Result<(), String> {
    let mut store_guard = state.lock().unwrap();
    if let Some(progress) = store_guard.get_mut(&scan_id) {
        if let Some(idx) = progress.pending_confirmations.iter().position(|c| c.id == confirmation_id) {
            let confirmation = progress.pending_confirmations.remove(idx);

            if remember {
                let mut cfg = config::AppConfig::load(&app_handle);
                if allow {
                    if !cfg.scan_rules.contains(&confirmation.path) {
                        cfg.scan_rules.push(confirmation.path.clone());
                    }
                } else {
                    if !cfg.skip_rules.contains(&confirmation.path) {
                        cfg.skip_rules.push(confirmation.path.clone());
                    }
                }
                let _ = cfg.save(&app_handle);
            }

            if allow {
                // Send directory to search workers via work channel
                let channels = channel_state.lock().unwrap();
                if let Some(work_tx) = channels.get(&scan_id) {
                    let _ = work_tx.send(DirWork {
                        path: PathBuf::from(&confirmation.path),
                    });
                }
            } else {
                progress.skipped_dirs.push(SkippedDir {
                    path: confirmation.path,
                    reason: if remember { "user_skip_remembered".to_string() } else { "user_skip".to_string() },
                });
            }
        }
    }
    Ok(())
}
