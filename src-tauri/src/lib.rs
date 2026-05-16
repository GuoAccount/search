use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

mod scanner;
use scanner::{ScanResult, ScanType, ScanCallback};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanConfig {
    pub path: String,
    pub keyword: String,
    pub scan_types: Vec<ScanType>,
    pub file_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanProgress {
    pub scan_id: String,
    pub status: String,
    pub files_scanned: u32,
    pub results_found: u32,
    pub current_path: String,
    pub results: Vec<ScanResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilePreview {
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_type: String,
    pub match_line: Option<u32>,
    pub context_lines: Vec<ContextLine>,
    pub match_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextLine {
    pub line_number: u32,
    pub content: String,
    pub is_match: bool,
}

type ScanStore = Arc<Mutex<HashMap<String, ScanProgress>>>;

#[tauri::command]
fn select_directory() -> Result<String, String> {
    Ok(String::new())
}

#[tauri::command]
async fn start_scan(
    config: ScanConfig,
    state: tauri::State<'_, ScanStore>,
) -> Result<String, String> {
    let scan_id = Uuid::new_v4().to_string();
    
    let progress = ScanProgress {
        scan_id: scan_id.clone(),
        status: "scanning".to_string(),
        files_scanned: 0,
        results_found: 0,
        current_path: String::new(),
        results: Vec::new(),
    };
    
    state.lock().unwrap().insert(scan_id.clone(), progress);
    
    let store = state.inner().clone();
    let sid = scan_id.clone();
    let should_cancel = Arc::new(Mutex::new(false));
    let should_cancel_clone = should_cancel.clone();
    
    // Store cancel flag
    let cancel_store = state.inner().clone();
    let cancel_sid = sid.clone();
    
    tokio::spawn(async move {
        let store_for_result = store.clone();
        let store_for_progress = store.clone();
        let sid_for_result = sid.clone();
        let sid_for_progress = sid.clone();
        
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
            should_cancel: should_cancel_clone,
        };
        
        scanner::scan_directory_with_callback(config, callback);
        
        let mut store_guard = store.lock().unwrap();
        if let Some(progress) = store_guard.get_mut(&sid) {
            progress.status = "completed".to_string();
            progress.current_path = String::new();
        }
    });
    
    Ok(scan_id)
}

#[tauri::command]
fn get_scan_progress(
    scan_id: String,
    state: tauri::State<'_, ScanStore>,
) -> Result<ScanProgress, String> {
    let store = state.lock().unwrap();
    store.get(&scan_id).cloned().ok_or_else(|| "Scan not found".to_string())
}

#[tauri::command]
fn cancel_scan(
    scan_id: String,
    state: tauri::State<'_, ScanStore>,
) -> Result<(), String> {
    let mut store = state.lock().unwrap();
    if let Some(progress) = store.get_mut(&scan_id) {
        progress.status = "cancelled".to_string();
    }
    Ok(())
}

#[tauri::command]
async fn read_file_preview(
    file_path: String,
    match_line: Option<u32>,
    context_lines: u32,
) -> Result<FilePreview, String> {
    let path = PathBuf::from(&file_path);
    let file_name = path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
    let file_size = metadata.len();
    
    let extension = path.extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    let file_type = match extension.as_str() {
        "txt" | "md" | "csv" | "json" | "xml" | "yaml" | "yml" | "toml" => "text",
        "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "go" | "java" | "c" | "cpp" | "h" => "code",
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" => "image",
        "pdf" => "pdf",
        _ => "other",
    }.to_string();
    
    let mut context = Vec::new();
    
    if file_type == "text" || file_type == "code" {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let lines: Vec<&str> = content.lines().collect();
        
        let center = match_line.unwrap_or(1) as usize;
        let start = if center > context_lines as usize {
            center - context_lines as usize
        } else {
            1
        };
        let end = (center + context_lines as usize).min(lines.len());
        
        for i in start..=end {
            context.push(ContextLine {
                line_number: i as u32,
                content: lines.get(i - 1).unwrap_or(&"").to_string(),
                is_match: i == center,
            });
        }
    }
    
    Ok(FilePreview {
        file_path,
        file_name,
        file_size,
        file_type,
        match_line,
        context_lines: context,
        match_type: "content".to_string(),
    })
}

#[tauri::command]
async fn move_to_trash(file_paths: Vec<String>) -> Result<u32, String> {
    let mut count = 0;
    for path in file_paths {
        let p = PathBuf::from(&path);
        if trash::delete(&p).is_ok() {
            count += 1;
        }
    }
    
    if count > 0 {
        play_trash_sound();
    }
    
    Ok(count)
}

fn play_trash_sound() {
    let possible_paths = vec![
        std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("Resources/resources/trash.aif")),
        std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("resources/trash.aif")),
    ];
    
    let resource_path = possible_paths.into_iter()
        .flatten()
        .find(|p| p.exists());
    
    if let Some(path) = resource_path {
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("afplay")
                .arg(path.to_string_lossy().to_string())
                .spawn();
        }
        
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("powershell")
                .args(["-c", &format!("(New-Object Media.SoundPlayer '{}').Play()", path.to_string_lossy())])
                .spawn();
        }
        
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("aplay")
                .arg(path.to_string_lossy().to_string())
                .spawn();
        }
    }
}

#[tauri::command]
fn get_file_info(file_path: String) -> Result<HashMap<String, String>, String> {
    let path = PathBuf::from(&file_path);
    let mut info = HashMap::new();
    
    let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
    
    info.insert("name".to_string(), path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string());
    info.insert("path".to_string(), file_path);
    info.insert("size".to_string(), metadata.len().to_string());
    info.insert("is_dir".to_string(), metadata.is_dir().to_string());
    
    if let Ok(time) = metadata.modified() {
        let datetime: chrono::DateTime<chrono::Local> = time.into();
        info.insert("modified".to_string(), datetime.format("%Y-%m-%d %H:%M:%S").to_string());
    }
    
    Ok(info)
}

#[tauri::command]
async fn reveal_in_finder(file_path: String) -> Result<(), String> {
    let path = PathBuf::from(&file_path);
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path.parent().unwrap_or(&path).to_string_lossy().to_string())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
async fn read_image_as_base64(file_path: String) -> Result<String, String> {
    let path = PathBuf::from(&file_path);
    let data = fs::read(&path).map_err(|e| e.to_string())?;
    
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
    
    let mime = match path.extension().unwrap_or_default().to_string_lossy().to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    };
    
    Ok(format!("data:{};base64,{}", mime, encoded))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let scan_store: ScanStore = Arc::new(Mutex::new(HashMap::new()));
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(scan_store)
        .invoke_handler(tauri::generate_handler![
            select_directory,
            start_scan,
            get_scan_progress,
            cancel_scan,
            read_file_preview,
            move_to_trash,
            get_file_info,
            reveal_in_finder,
            read_image_as_base64
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
