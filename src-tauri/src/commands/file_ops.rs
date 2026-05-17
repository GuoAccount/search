use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::types::{ContextLine, FilePreview, ScanStore};

#[tauri::command]
pub async fn read_file_preview(
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
pub async fn move_to_trash(
    file_paths: Vec<String>,
    scan_id: Option<String>,
    state: tauri::State<'_, ScanStore>,
) -> Result<u32, String> {
    let mut count = 0;
    for path in &file_paths {
        let p = PathBuf::from(path);
        if trash::delete(&p).is_ok() {
            count += 1;
        }
    }

    // Remove deleted files from scan store to prevent polling from resurrecting them
    if let Some(sid) = scan_id {
        if count > 0 {
            let mut store = state.lock().unwrap();
            if let Some(progress) = store.get_mut(&sid) {
                progress.results.retain(|r| !file_paths.contains(&r.file_path));
                progress.results_found = progress.results.len() as u32;
            }
        }
    }
    
    if count > 0 {
        crate::commands::system::play_trash_sound();
    }
    
    Ok(count)
}

#[tauri::command]
pub fn get_file_info(file_path: String) -> Result<HashMap<String, String>, String> {
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
pub async fn reveal_in_finder(file_path: String) -> Result<(), String> {
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
pub async fn read_image_as_base64(file_path: String) -> Result<String, String> {
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
