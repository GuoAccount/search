use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;
use exif::Reader;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScanType {
    FileName,
    TextContent,
    ExifData,
    OcrText,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanResult {
    pub file_path: String,
    pub file_name: String,
    pub match_type: String,
    pub match_line: Option<u32>,
    pub match_context: Option<String>,
    pub file_size: u64,
    pub file_extension: String,
    pub is_dir: bool,
}

pub struct ScanCallback {
    pub on_result: Box<dyn Fn(ScanResult) + Send>,
    pub on_progress: Box<dyn Fn(u32, String) + Send>,
    pub should_cancel: Arc<Mutex<bool>>,
}

pub fn scan_directory_with_callback(config: crate::ScanConfig, callback: ScanCallback) {
    let path = config.path.clone();
    let keyword = config.keyword.to_lowercase();
    let scan_types = config.scan_types;
    let exclude_patterns = config.exclude_patterns;
    let file_extensions: Vec<String> = config.file_extensions.iter().map(|e| e.to_lowercase()).collect();
    
    // First collect all entries
    let entries: Vec<_> = WalkDir::new(&path)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| !is_excluded(e.path(), &exclude_patterns))
        .filter_map(|e| e.ok())
        .collect();
    
    let total = entries.len() as u32;
    
    // Process entries sequentially for real-time updates
    for (index, entry) in entries.iter().enumerate() {
        // Check if cancelled
        if *callback.should_cancel.lock().unwrap() {
            break;
        }
        
        let path = entry.path();
        let file_name = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        // Update progress
        let current_dir = path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        (callback.on_progress)((index as u32) + 1, current_dir);
        
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        
        let extension = path.extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
            .to_lowercase();
        
        // Skip if extension not in allowed list
        if !metadata.is_dir() && !file_extensions.is_empty() && !file_extensions.contains(&extension) {
            continue;
        }
        
        // File name matching
        if scan_types.contains(&ScanType::FileName) {
            if file_name.to_lowercase().contains(&keyword) {
                (callback.on_result)(ScanResult {
                    file_path: path.to_string_lossy().to_string(),
                    file_name: file_name.clone(),
                    match_type: "filename".to_string(),
                    match_line: None,
                    match_context: Some(file_name.clone()),
                    file_size: metadata.len(),
                    file_extension: extension.clone(),
                    is_dir: metadata.is_dir(),
                });
            }
        }
        
        // Text content matching
        if scan_types.contains(&ScanType::TextContent) && !metadata.is_dir() {
            if is_text_file(&extension) {
                if let Ok(content) = fs::read_to_string(path) {
                    for (line_num, line) in content.lines().enumerate() {
                        if line.to_lowercase().contains(&keyword) {
                            (callback.on_result)(ScanResult {
                                file_path: path.to_string_lossy().to_string(),
                                file_name: file_name.clone(),
                                match_type: "content".to_string(),
                                match_line: Some((line_num + 1) as u32),
                                match_context: Some(line.to_string()),
                                file_size: metadata.len(),
                                file_extension: extension.clone(),
                                is_dir: false,
                            });
                            break;
                        }
                    }
                }
            }
        }
        
        // EXIF data matching
        if scan_types.contains(&ScanType::ExifData) && !metadata.is_dir() {
            if is_image_file(&extension) {
                if let Ok(exif_data) = extract_exif(path) {
                    if exif_data.to_lowercase().contains(&keyword) {
                        (callback.on_result)(ScanResult {
                            file_path: path.to_string_lossy().to_string(),
                            file_name: file_name.clone(),
                            match_type: "exif".to_string(),
                            match_line: None,
                            match_context: Some(exif_data),
                            file_size: metadata.len(),
                            file_extension: extension.clone(),
                            is_dir: false,
                        });
                    }
                }
            }
        }
        
        // OCR text matching (macOS only)
        #[cfg(target_os = "macos")]
        if scan_types.contains(&ScanType::OcrText) && !metadata.is_dir() {
            if is_image_file(&extension) {
                if let Ok(ocr_text) = perform_ocr(path) {
                    if !ocr_text.is_empty() && ocr_text.to_lowercase().contains(&keyword) {
                        (callback.on_result)(ScanResult {
                            file_path: path.to_string_lossy().to_string(),
                            file_name: file_name.clone(),
                            match_type: "ocr".to_string(),
                            match_line: None,
                            match_context: Some(ocr_text),
                            file_size: metadata.len(),
                            file_extension: extension.clone(),
                            is_dir: false,
                        });
                    }
                }
            }
        }
    }
}

fn is_excluded(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    let file_name = path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    
    if path_str.contains("/.") || path_str.contains("\\.") {
        return true;
    }
    if file_name.starts_with('.') {
        return true;
    }
    
    for pattern in patterns {
        let p = pattern.to_lowercase();
        if path_str.contains(&p) || file_name.contains(&p) {
            return true;
        }
    }
    
    false
}

fn is_text_file(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "txt" | "md" | "csv" | "json" | "xml" | "yaml" | "yml" | "toml" |
        "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "go" | "java" | "c" | "cpp" | "h" |
        "html" | "css" | "scss" | "less" | "sh" | "bash" | "zsh" | "fish" |
        "env" | "gitignore" | "dockerignore" | "makefile" | "cmake" |
        "sql" | "graphql" | "proto" | "ini" | "cfg" | "conf" | "config"
    )
}

fn is_image_file(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff" | "tif" | "heic" | "heif"
    )
}

fn extract_exif(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut bufreader = std::io::BufReader::new(file);
    
    let exif = Reader::new().read_from_container(&mut bufreader)
        .map_err(|e| e.to_string())?;
    
    let mut exif_data = Vec::new();
    
    for field in exif.fields() {
        let tag = field.tag.to_string();
        let value = field.display_value().to_string();
        exif_data.push(format!("{}: {}", tag, value));
    }
    
    Ok(exif_data.join("; "))
}

fn perform_ocr(path: &Path) -> Result<String, String> {
    let script = include_str!("../resources/ocr.swift");
    
    let temp_dir = std::env::temp_dir();
    let temp_script = temp_dir.join("filescope_ocr.swift");
    fs::write(&temp_script, script).map_err(|e| e.to_string())?;
    
    let output = Command::new("swift")
        .arg(&temp_script)
        .arg(path.to_string_lossy().to_string())
        .output()
        .map_err(|e| e.to_string())?;
    
    let _ = fs::remove_file(&temp_script);
    
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(text.trim().to_string())
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        Err(error)
    }
}
