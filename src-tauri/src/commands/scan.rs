use std::sync::{Arc, Mutex};
use tauri::Emitter;
use uuid::Uuid;

use crate::config;
use crate::scanner::{ScanCallback, ScanResult};
use crate::types::{PendingConfirmation, ScanConfig, ScanProgress, SkippedDir, PauseStore, CancelStore, ScanStore};

#[tauri::command]
pub async fn start_scan(
    app_handle: tauri::AppHandle,
    config: ScanConfig,
    state: tauri::State<'_, ScanStore>,
    pause_state: tauri::State<'_, PauseStore>,
    cancel_state: tauri::State<'_, CancelStore>,
) -> Result<String, String> {
    let scan_id = Uuid::new_v4().to_string();
    
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
    
    let store = state.inner().clone();
    let sid = scan_id.clone();
    let should_cancel = Arc::new(Mutex::new(false));
    let should_cancel_clone = should_cancel.clone();
    let should_pause = Arc::new(Mutex::new(false));
    let should_pause_clone = should_pause.clone();
    
    // Store pause flag
    pause_state.0.lock().unwrap().insert(scan_id.clone(), should_pause.clone());
    
    // Store cancel flag
    cancel_state.0.lock().unwrap().insert(scan_id.clone(), should_cancel.clone());
    
    // Load app config
    let app_config = config::AppConfig::load(&app_handle);
    
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
                // Emit event to frontend
                let _ = app_handle_clone.emit("confirmation-needed", confirmation);
            }),
            on_dir_skipped: Box::new(move |skipped: SkippedDir| {
                let mut store_guard = store_for_skipped.lock().unwrap();
                if let Some(progress) = store_guard.get_mut(&sid_for_skipped) {
                    progress.skipped_dirs.push(skipped);
                }
            }),
            should_cancel: should_cancel_clone,
            should_pause: should_pause_clone,
        };
        
        crate::scanner::scan_directory_with_callback(config, app_config, callback);
        
        let mut store_guard = store.lock().unwrap();
        if let Some(progress) = store_guard.get_mut(&sid) {
            progress.status = "completed".to_string();
            progress.current_path = String::new();
        }
    });
    
    Ok(scan_id)
}

#[tauri::command]
pub async fn scan_sub_directory(
    scan_id: String,
    directory_path: String,
    config: ScanConfig,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, ScanStore>,
    pause_state: tauri::State<'_, PauseStore>,
    cancel_state: tauri::State<'_, CancelStore>,
) -> Result<String, String> {
    let sub_scan_id = Uuid::new_v4().to_string();
    let sub_scan_id_clone = sub_scan_id.clone();
    
    // Create sub-scan progress
    let progress = ScanProgress {
        scan_id: sub_scan_id.clone(),
        parent_scan_id: Some(scan_id.clone()),
        status: "scanning".to_string(),
        files_scanned: 0,
        results_found: 0,
        current_path: String::new(),
        results: Vec::new(),
        pending_confirmations: Vec::new(),
        skipped_dirs: Vec::new(),
    };
    
    state.lock().unwrap().insert(sub_scan_id.clone(), progress);
    
    let store = state.inner().clone();
    let should_cancel = Arc::new(Mutex::new(false));
    let should_cancel_clone = should_cancel.clone();
    let should_pause = Arc::new(Mutex::new(false));
    let should_pause_clone = should_pause.clone();
    
    // Store pause flag
    pause_state.0.lock().unwrap().insert(sub_scan_id.clone(), should_pause.clone());
    
    // Store cancel flag
    cancel_state.0.lock().unwrap().insert(sub_scan_id.clone(), should_cancel.clone());
    
    // Load app config
    let app_config = config::AppConfig::load(&app_handle);
    
    // Create sub-scan config
    let sub_config = ScanConfig {
        path: directory_path,
        keyword: config.keyword,
        scan_types: config.scan_types,
        file_extensions: config.file_extensions,
        exclude_patterns: config.exclude_patterns,
    };
    
    tokio::spawn(async move {
        let store_for_result = store.clone();
        let store_for_progress = store.clone();
        let store_for_confirmation = store.clone();
        let store_for_skipped = store.clone();
        let sid_for_result = sub_scan_id_clone.clone();
        let sid_for_progress = sub_scan_id_clone.clone();
        let sid_for_confirmation = sub_scan_id_clone.clone();
        let sid_for_skipped = sub_scan_id_clone.clone();
        let app_handle_clone = app_handle.clone();
        let main_scan_id = scan_id.clone();
        
        let callback = ScanCallback {
            on_result: Box::new(move |result: ScanResult| {
                let mut store_guard = store_for_result.lock().unwrap();
                // Add to sub-scan results
                if let Some(progress) = store_guard.get_mut(&sid_for_result) {
                    progress.results.push(result.clone());
                    progress.results_found = progress.results.len() as u32;
                }
                // Also add to main scan results
                if let Some(main_progress) = store_guard.get_mut(&main_scan_id) {
                    main_progress.results.push(result);
                    main_progress.results_found = main_progress.results.len() as u32;
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
            should_cancel: should_cancel_clone,
            should_pause: should_pause_clone,
        };
        
        crate::scanner::scan_directory_with_callback(sub_config, app_config, callback);
        
        let mut store_guard = store.lock().unwrap();
        if let Some(progress) = store_guard.get_mut(&sub_scan_id_clone) {
            progress.status = "completed".to_string();
            progress.current_path = String::new();
        }
    });
    
    Ok(sub_scan_id)
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
    // Update status
    let mut store = state.lock().unwrap();
    if let Some(progress) = store.get_mut(&scan_id) {
        progress.status = "cancelled".to_string();
    }
    
    // Set cancel flag
    let cancel_store = cancel_state.0.lock().unwrap();
    if let Some(cancel_flag) = cancel_store.get(&scan_id) {
        *cancel_flag.lock().unwrap() = true;
    }
    
    Ok(())
}

#[tauri::command]
pub fn pause_scan(
    scan_id: String,
    state: tauri::State<'_, ScanStore>,
    pause_state: tauri::State<'_, PauseStore>,
) -> Result<(), String> {
    // Update status
    let mut store = state.lock().unwrap();
    if let Some(progress) = store.get_mut(&scan_id) {
        progress.status = "paused".to_string();
    }
    
    // Set pause flag
    let pause_store = pause_state.0.lock().unwrap();
    if let Some(pause_flag) = pause_store.get(&scan_id) {
        *pause_flag.lock().unwrap() = true;
    }
    
    Ok(())
}

#[tauri::command]
pub fn resume_scan(
    scan_id: String,
    state: tauri::State<'_, ScanStore>,
    pause_state: tauri::State<'_, PauseStore>,
) -> Result<(), String> {
    // Update status
    let mut store = state.lock().unwrap();
    if let Some(progress) = store.get_mut(&scan_id) {
        progress.status = "scanning".to_string();
    }
    
    // Clear pause flag
    let pause_store = pause_state.0.lock().unwrap();
    if let Some(pause_flag) = pause_store.get(&scan_id) {
        *pause_flag.lock().unwrap() = false;
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
) -> Result<(), String> {
    let mut store_guard = state.lock().unwrap();
    if let Some(progress) = store_guard.get_mut(&scan_id) {
        // Find and remove the confirmation
        if let Some(idx) = progress.pending_confirmations.iter().position(|c| c.id == confirmation_id) {
            let confirmation = progress.pending_confirmations.remove(idx);
            
            // If remember, update config
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
            
            // If not allow, add to skipped
            if !allow {
                progress.skipped_dirs.push(SkippedDir {
                    path: confirmation.path,
                    reason: if remember { "user_skip_remembered".to_string() } else { "user_skip".to_string() },
                });
            }
        }
    }
    Ok(())
}
