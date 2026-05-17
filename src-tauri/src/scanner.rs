use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use uuid::Uuid;
use exif::Reader;

use crate::types::{PendingConfirmation, SkippedDir, ScanConfig};

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
    pub on_confirmation_needed: Box<dyn Fn(PendingConfirmation) + Send>,
    pub on_dir_skipped: Box<dyn Fn(SkippedDir) + Send>,
    pub should_cancel: Arc<Mutex<bool>>,
    pub should_pause: Arc<Mutex<bool>>,
}

#[derive(Clone)]
struct ScanContext {
    keyword: String,
    scan_types: Vec<String>,
    file_extensions: Vec<String>,
    exclude_patterns: Vec<String>,
    skip_rules: Vec<String>,
    scan_rules: Vec<String>,
    threshold: u64,
    ask_on_large_dir: bool,
}

enum ScanMessage {
    Result(ScanResult),
    Progress(u32, String),
}

pub fn scan_directory_with_callback(
    config: ScanConfig,
    app_config: crate::config::AppConfig,
    callback: ScanCallback,
) {
    let path = config.path.clone();
    let keyword = config.keyword.to_lowercase();
    let scan_types = config.scan_types;
    let exclude_patterns = config.exclude_patterns;
    let file_extensions: Vec<String> = config.file_extensions.iter().map(|e| e.to_lowercase()).collect();
    
    let skip_rules = app_config.skip_rules.clone();
    let scan_rules = app_config.scan_rules.clone();
    let threshold = app_config.scan.large_dir_threshold;
    let ask_on_large_dir = app_config.scan.ask_on_large_dir;
    
    let ctx = ScanContext {
        keyword,
        scan_types,
        file_extensions,
        exclude_patterns,
        skip_rules,
        scan_rules,
        threshold,
        ask_on_large_dir,
    };
    
    // Collect all entries using BFS with filtering
    let entries = collect_entries_bfs(
        Path::new(&path),
        &ctx,
        &callback,
    );
    
    let files_scanned = Arc::new(Mutex::new(0u32));
    let should_cancel = callback.should_cancel.clone();
    let should_pause = callback.should_pause.clone();
    
    // Use channel to collect results from parallel processing
    let (tx, rx) = mpsc::channel::<ScanMessage>();
    
    // Spawn a thread to handle results
    let on_result = callback.on_result;
    let on_progress = callback.on_progress;
    let result_handler = std::thread::spawn(move || {
        for msg in rx {
            match msg {
                ScanMessage::Result(result) => (on_result)(result),
                ScanMessage::Progress(count, path) => (on_progress)(count, path),
            }
        }
    });
    
    // Process files in parallel using rayon with chunks
    let num_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let chunk_size = (entries.len() / num_threads).max(1);
    
    entries.par_chunks(chunk_size).for_each(|chunk| {
        let tx = tx.clone();
        let files_scanned = files_scanned.clone();
        let should_cancel = should_cancel.clone();
        let should_pause = should_pause.clone();
        let ctx = ctx.clone();
        
        for entry_path in chunk {
            // Check cancel
            if *should_cancel.lock().unwrap() {
                return;
            }
            
            // Check pause - busy wait
            while *should_pause.lock().unwrap() {
                std::thread::sleep(std::time::Duration::from_millis(100));
                // Check cancel while paused
                if *should_cancel.lock().unwrap() {
                    return;
                }
            }
            
            let metadata = match fs::metadata(entry_path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            
            let file_name = entry_path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            
            let extension = entry_path.extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
                .to_lowercase();
            
            // Skip if extension not in allowed list (for files)
            if !metadata.is_dir() && !ctx.file_extensions.is_empty() && !ctx.file_extensions.contains(&extension) {
                continue;
            }
            
            // Update progress
            {
                let mut count = files_scanned.lock().unwrap();
                *count += 1;
                let current_dir = entry_path.parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                let _ = tx.send(ScanMessage::Progress(*count, current_dir));
            }
            
            // File name matching
            if ctx.scan_types.contains(&"file_name".to_string()) {
                if file_name.to_lowercase().contains(&ctx.keyword) {
                    let _ = tx.send(ScanMessage::Result(ScanResult {
                        file_path: entry_path.to_string_lossy().to_string(),
                        file_name: file_name.clone(),
                        match_type: "filename".to_string(),
                        match_line: None,
                        match_context: Some(file_name.clone()),
                        file_size: metadata.len(),
                        file_extension: extension.clone(),
                        is_dir: metadata.is_dir(),
                    }));
                }
            }
            
            // Text content matching
            if ctx.scan_types.contains(&"text_content".to_string()) && !metadata.is_dir() {
                if is_text_file(&extension) {
                    if let Ok(content) = fs::read_to_string(entry_path) {
                        for (line_num, line) in content.lines().enumerate() {
                            if line.to_lowercase().contains(&ctx.keyword) {
                                let _ = tx.send(ScanMessage::Result(ScanResult {
                                    file_path: entry_path.to_string_lossy().to_string(),
                                    file_name: file_name.clone(),
                                    match_type: "content".to_string(),
                                    match_line: Some((line_num + 1) as u32),
                                    match_context: Some(line.to_string()),
                                    file_size: metadata.len(),
                                    file_extension: extension.clone(),
                                    is_dir: false,
                                }));
                                break;
                            }
                        }
                    }
                }
            }
            
            // EXIF data matching
            if ctx.scan_types.contains(&"exif_data".to_string()) && !metadata.is_dir() {
                if is_image_file(&extension) {
                    if let Ok(exif_data) = extract_exif(entry_path) {
                        if exif_data.to_lowercase().contains(&ctx.keyword) {
                            let _ = tx.send(ScanMessage::Result(ScanResult {
                                file_path: entry_path.to_string_lossy().to_string(),
                                file_name: file_name.clone(),
                                match_type: "exif".to_string(),
                                match_line: None,
                                match_context: Some(exif_data),
                                file_size: metadata.len(),
                                file_extension: extension.clone(),
                                is_dir: false,
                            }));
                        }
                    }
                }
            }
            
            // OCR text matching (macOS only)
            #[cfg(target_os = "macos")]
            if ctx.scan_types.contains(&"ocr_text".to_string()) && !metadata.is_dir() {
                if is_image_file(&extension) {
                    if let Ok(ocr_text) = perform_ocr(entry_path) {
                        if !ocr_text.is_empty() && ocr_text.to_lowercase().contains(&ctx.keyword) {
                            let _ = tx.send(ScanMessage::Result(ScanResult {
                                file_path: entry_path.to_string_lossy().to_string(),
                                file_name: file_name.clone(),
                                match_type: "ocr".to_string(),
                                match_line: None,
                                match_context: Some(ocr_text),
                                file_size: metadata.len(),
                                file_extension: extension.clone(),
                                is_dir: false,
                            }));
                        }
                    }
                }
            }
        }
    });
    
    // Drop all senders
    drop(tx);
    
    // Wait for result handler to finish
    result_handler.join().unwrap();
}

fn collect_entries_bfs(
    root: &Path,
    ctx: &ScanContext,
    callback: &ScanCallback,
) -> Vec<PathBuf> {
    let mut entries = Vec::new();
    let mut queue = VecDeque::new();
    
    // Start with root directory
    if let Ok(read_dir) = fs::read_dir(root) {
        for entry in read_dir.filter_map(|e| e.ok()) {
            queue.push_back(entry.path());
        }
    }
    
    while let Some(current_path) = queue.pop_front() {
        // Check cancel
        if *callback.should_cancel.lock().unwrap() {
            break;
        }
        
        // Check if should process
        if !should_process_entry(&current_path, ctx, callback) {
            continue;
        }
        
        let is_dir = current_path.is_dir();
        
        // If directory, add its children to queue
        if is_dir {
            if let Ok(read_dir) = fs::read_dir(&current_path) {
                for entry in read_dir.filter_map(|e| e.ok()) {
                    queue.push_back(entry.path());
                }
            }
        }
        
        entries.push(current_path);
    }
    
    entries
}

fn should_process_entry(
    path: &Path,
    ctx: &ScanContext,
    callback: &ScanCallback,
) -> bool {
    // Check if hidden file/dir
    let path_str = path.to_string_lossy();
    let file_name = path.file_name()
        .unwrap_or_default()
        .to_string_lossy();
    
    if path_str.contains("/.") || path_str.contains("\\.") || file_name.starts_with('.') {
        return false;
    }
    
    // Check exclude patterns
    for pattern in &ctx.exclude_patterns {
        let p = pattern.to_lowercase();
        if path_str.to_lowercase().contains(&p) || file_name.to_lowercase().contains(&p) {
            return false;
        }
    }
    
    // Check scan_rules (force scan, highest priority)
    if matches_rules(path, &ctx.scan_rules) {
        return true;
    }
    
    // Check skip_rules (force skip)
    if matches_rules(path, &ctx.skip_rules) {
        if path.is_dir() {
            (callback.on_dir_skipped)(SkippedDir {
                path: path.to_string_lossy().to_string(),
                reason: "rule".to_string(),
            });
        }
        return false;
    }
    
    // For files, always process
    if !path.is_dir() {
        return true;
    }
    
    // For directories, do a quick check (only count direct children, not recursive)
    if ctx.ask_on_large_dir {
        let entry_count = count_entries_fast(path);
        if entry_count > ctx.threshold {
            let confirmation = PendingConfirmation {
                id: Uuid::new_v4().to_string(),
                path: path.to_string_lossy().to_string(),
                entry_count,
            };
            (callback.on_confirmation_needed)(confirmation);
            (callback.on_dir_skipped)(SkippedDir {
                path: path.to_string_lossy().to_string(),
                reason: "large_dir".to_string(),
            });
            return false;
        }
    }
    
    true
}

fn matches_rules(path: &Path, rules: &[String]) -> bool {
    let file_name = path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let path_str = path.to_string_lossy().to_string();
    
    rules.iter().any(|rule| {
        file_name == *rule || path_str.contains(rule.as_str())
    })
}

/// Fast entry count - only counts direct children, not recursive
fn count_entries_fast(path: &Path) -> u64 {
    fs::read_dir(path)
        .map(|entries| entries.filter_map(|e| e.ok()).count() as u64)
        .unwrap_or(0)
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
