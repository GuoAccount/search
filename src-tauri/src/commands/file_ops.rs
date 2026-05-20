use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::types::{ContextLine, FilePreview, ScanStore};
use crate::scanner;

const DEFAULT_CONTEXT_AROUND: usize = 100;

fn extract_context(line: &str, keyword: &str, context_around: usize) -> String {
    let lower = line.to_lowercase();
    let kw_lower = keyword.to_lowercase();
    if let Some(pos) = lower.find(&kw_lower) {
        let char_start: usize = line[..pos].chars().count();
        let kw_char_len: usize = keyword.chars().count();
        let total_chars: usize = line.chars().count();

        let ctx_before = char_start.saturating_sub(context_around);
        let ctx_after = (char_start + kw_char_len + context_around).min(total_chars);

        let chunk: String = line.chars().skip(ctx_before).take(ctx_after - ctx_before).collect();

        let mut result = String::new();
        if ctx_before > 0 {
            result.push_str("…");
        }
        result.push_str(&chunk);
        if ctx_after < total_chars {
            result.push_str("…");
        }
        result
    } else {
        line.chars().take(context_around * 2).collect()
    }
}

fn read_text_content(path: &PathBuf, extension: &str) -> Result<String, String> {
    if scanner::is_text_file(extension) {
        fs::read_to_string(path).map_err(|e| e.to_string())
    } else if scanner::is_document_file(extension) {
        scanner::extract_document_text(path, extension)
    } else {
        fs::read_to_string(path).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn read_file_preview(
    file_path: String,
    match_line: Option<u32>,
    match_type: Option<String>,
    context_lines: u32,
    keyword: Option<String>,
    context_length: Option<u32>,
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

    let mut context = Vec::new();

    if let Ok(content) = read_text_content(&path, &extension) {
        let lines: Vec<&str> = content.lines().collect();
        let kw = keyword.as_deref().unwrap_or("");

        let center = if let Some(line_num) = match_line {
            line_num as usize
        } else if !kw.is_empty() {
            let kw_lower = kw.to_lowercase();
            lines.iter().position(|l| l.to_lowercase().contains(&kw_lower))
                .map(|i| i + 1)
                .unwrap_or(1)
        } else {
            1
        };

        let start = center.saturating_sub(context_lines as usize).max(1);
        let end = (center + context_lines as usize).min(lines.len());
        let ctx_len = context_length.unwrap_or(DEFAULT_CONTEXT_AROUND as u32) as usize;

        for i in start..=end {
            let raw = lines.get(i - 1).unwrap_or(&"");
            let content = if i == center && !kw.is_empty() {
                extract_context(raw, kw, ctx_len)
            } else {
                raw.to_string()
            };
            context.push(ContextLine {
                line_number: i as u32,
                content,
                is_match: i == center,
            });
        }
    }

    Ok(FilePreview {
        file_path,
        file_name,
        file_size,
        file_type: extension,
        match_line,
        context_lines: context,
        match_type: match_type.unwrap_or_else(|| "content".to_string()),
    })
}

#[tauri::command]
pub async fn move_to_trash(
    app_handle: tauri::AppHandle,
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
        crate::commands::system::play_trash_sound(&app_handle);
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
pub async fn reveal_in_finder(app_handle: tauri::AppHandle, file_path: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let path = PathBuf::from(&file_path);
    app_handle.opener()
        .reveal_item_in_dir(&path)
        .map_err(|e| e.to_string())?;
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
