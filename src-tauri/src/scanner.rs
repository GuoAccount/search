use rayon;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use uuid::Uuid;
use exif::Reader;

use crate::types::{PendingConfirmation, SkippedDir, ScanConfig, DirWork};

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

// ─── Public entry point ───────────────────────────────────────────────────────

pub fn scan_directory(
    config: ScanConfig,
    app_config: crate::config::AppConfig,
    callback: ScanCallback,
    work_tx: Sender<DirWork>,
    work_rx: Receiver<DirWork>,
) {
    let root = PathBuf::from(&config.path);
    let ctx = Arc::new(ScanContext {
        keyword: config.keyword.to_lowercase(),
        scan_types: config.scan_types,
        file_extensions: config.file_extensions.iter().map(|e| e.to_lowercase()).collect(),
        exclude_patterns: config.exclude_patterns,
        skip_rules: app_config.skip_rules.clone(),
        scan_rules: app_config.scan_rules.clone(),
        threshold: app_config.scan.large_dir_threshold,
        ask_on_large_dir: app_config.scan.ask_on_large_dir,
    });

    let (result_tx, result_rx) = mpsc::channel::<ScanResult>();
    let (progress_tx, progress_rx) = mpsc::channel::<(u32, String)>();

    let cancel_flag = callback.should_cancel.clone();
    let pause_flag = callback.should_pause.clone();
    let files_scanned = Arc::new(AtomicU32::new(0));

    // 1. BFS thread: traverses tree, classifies directories
    let bfs_ctx = ctx.clone();
    let bfs_cancel = cancel_flag.clone();
    let bfs_pause = pause_flag.clone();
    let bfs_work_tx = work_tx.clone();
    let bfs_handle = std::thread::spawn(move || {
        bfs_scan(
            &root,
            &bfs_ctx,
            bfs_work_tx,
            &*callback.on_confirmation_needed,
            &*callback.on_dir_skipped,
            &bfs_cancel,
            &bfs_pause,
        );
    });

    // 2. Result handler thread
    let result_handle = std::thread::spawn(move || {
        for result in result_rx {
            (callback.on_result)(result);
        }
    });

    // 3. Progress handler thread
    let progress_handle = std::thread::spawn(move || {
        for (count, path) in progress_rx {
            (callback.on_progress)(count, path);
        }
    });

    // 4. Dispatcher: reads work channel, dispatches to rayon thread pool
    let (done_tx, done_rx) = mpsc::channel::<()>();
    let done_tx = Arc::new(Mutex::new(done_tx));

    let rtx_main = result_tx.clone();
    let ptx_main = progress_tx.clone();

    let dispatch_done_tx = done_tx.clone();
    let dispatch_handle = std::thread::spawn(move || {
        let active_count = Arc::new(AtomicU32::new(0));

        for work in work_rx {
            if *cancel_flag.lock().unwrap() {
                break;
            }

            let ctx = ctx.clone();
            let rtx = rtx_main.clone();
            let ptx = ptx_main.clone();
            let fs = files_scanned.clone();
            let cf = cancel_flag.clone();
            let pf = pause_flag.clone();
            let active = active_count.clone();
            let done = done_tx.clone();

            active.fetch_add(1, Ordering::Relaxed);

            rayon::spawn(move || {
                search_directory(&work.path, &ctx, &rtx, &ptx, &fs, &cf, &pf);
                let prev = active.fetch_sub(1, Ordering::Relaxed);
                if prev == 1 {
                    let _ = done.lock().unwrap().send(());
                }
            });
        }

        if active_count.load(Ordering::Relaxed) == 0 {
            let _ = dispatch_done_tx.lock().unwrap().send(());
        }
    });

    // Wait for BFS to finish
    let _ = bfs_handle.join();
    drop(work_tx);
    let _ = dispatch_handle.join();

    // Wait for all rayon tasks to complete
    let _ = done_rx.recv();

    // 5. Cleanup result/progress handlers
    drop(result_tx);
    drop(progress_tx);
    let _ = result_handle.join();
    let _ = progress_handle.join();
}

// ─── BFS thread ──────────────────────────────────────────────────────────────

fn bfs_scan(
    root: &Path,
    ctx: &ScanContext,
    work_tx: Sender<DirWork>,
    on_confirmation_needed: &dyn Fn(PendingConfirmation),
    on_dir_skipped: &dyn Fn(SkippedDir),
    cancel_flag: &Arc<Mutex<bool>>,
    pause_flag: &Arc<Mutex<bool>>,
) {
    let mut queue = VecDeque::new();
    queue.push_back(root.to_path_buf());

    while let Some(dir) = queue.pop_front() {
        if *cancel_flag.lock().unwrap() {
            break;
        }
        check_pause(pause_flag, cancel_flag);

        let Ok(read_dir) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in read_dir.filter_map(|e| e.ok()) {
            if *cancel_flag.lock().unwrap() {
                return;
            }

            let path = entry.path();

            // Hidden check
            if is_hidden(&path) {
                continue;
            }

            // Exclude patterns
            if matches_exclude(&path, &ctx.exclude_patterns) {
                continue;
            }

            // scan_rules: force scan (highest priority)
            if matches_rules(&path, &ctx.scan_rules) {
                if path.is_dir() {
                    enqueue_dir(&path, &ctx, &work_tx, on_confirmation_needed, on_dir_skipped, cancel_flag, pause_flag, &mut queue);
                }
                // Files under scan_rules dirs will be handled when BFS sends the dir
                continue;
            }

            // skip_rules: force skip
            if matches_rules(&path, &ctx.skip_rules) {
                if path.is_dir() {
                    on_dir_skipped(SkippedDir {
                        path: path.to_string_lossy().to_string(),
                        reason: "rule".to_string(),
                    });
                }
                continue;
            }

            // Directories: classify by threshold
            if path.is_dir() {
                enqueue_dir(&path, &ctx, &work_tx, on_confirmation_needed, on_dir_skipped, cancel_flag, pause_flag, &mut queue);
            }
            // Files: skip (they'll be processed when their parent dir is sent to work_tx)
        }
    }
    // work_tx dropped here → signals no more BFS work
}

fn enqueue_dir(
    path: &Path,
    ctx: &ScanContext,
    work_tx: &Sender<DirWork>,
    on_confirmation_needed: &dyn Fn(PendingConfirmation),
    on_dir_skipped: &dyn Fn(SkippedDir),
    cancel_flag: &Arc<Mutex<bool>>,
    pause_flag: &Arc<Mutex<bool>>,
    queue: &mut VecDeque<PathBuf>,
) {
    let count = count_entries_fast(path);

    if count > ctx.threshold && ctx.ask_on_large_dir {
        // Over threshold: send confirmation request, do NOT enqueue
        on_confirmation_needed(PendingConfirmation {
            id: Uuid::new_v4().to_string(),
            path: path.to_string_lossy().to_string(),
            entry_count: count,
        });
        on_dir_skipped(SkippedDir {
            path: path.to_string_lossy().to_string(),
            reason: "large_dir".to_string(),
        });
    } else {
        // Under threshold or not asking: send work item and continue BFS
        let _ = work_tx.send(DirWork { path: path.to_path_buf() });
        queue.push_back(path.to_path_buf());
    }
}

// ─── Search worker: processes one directory ───────────────────────────────────

fn search_directory(
    dir: &Path,
    ctx: &ScanContext,
    result_tx: &Sender<ScanResult>,
    progress_tx: &Sender<(u32, String)>,
    files_scanned: &AtomicU32,
    cancel_flag: &Arc<Mutex<bool>>,
    pause_flag: &Arc<Mutex<bool>>,
) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };

    let dir_str = dir.to_string_lossy().to_string();

    for entry in read_dir.filter_map(|e| e.ok()) {
        if *cancel_flag.lock().unwrap() {
            return;
        }
        check_pause(pause_flag, cancel_flag);

        let path = entry.path();

        // Skip directories (BFS handles them)
        if path.is_dir() {
            continue;
        }

        // Skip hidden files
        if is_hidden(&path) {
            continue;
        }

        // Skip exclude patterns
        if matches_exclude(&path, &ctx.exclude_patterns) {
            continue;
        }

        // Skip rules (files matching skip_rules)
        if matches_rules(&path, &ctx.skip_rules) {
            continue;
        }

        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let extension = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
            .to_lowercase();

        let ext_allowed = ctx.file_extensions.is_empty() || ctx.file_extensions.contains(&extension);

        // Update progress
        let count = files_scanned.fetch_add(1, Ordering::Relaxed) + 1;
        let _ = progress_tx.send((count, dir_str.clone()));

        // File name matching (always runs, regardless of extension filter)
        if ctx.scan_types.contains(&"file_name".to_string()) {
            if file_name.to_lowercase().contains(&ctx.keyword) {
                let _ = result_tx.send(ScanResult {
                    file_path: path.to_string_lossy().to_string(),
                    file_name: file_name.clone(),
                    match_type: "filename".to_string(),
                    match_line: None,
                    match_context: Some(file_name.clone()),
                    file_size: metadata.len(),
                    file_extension: extension.clone(),
                    is_dir: false,
                });
            }
        }

        // Content-based matching (only for files with allowed extensions)
        if !ext_allowed {
            continue;
        }

        // Text content matching
        if ctx.scan_types.contains(&"text_content".to_string()) && is_text_file(&extension) {
            if let Ok(content) = fs::read_to_string(&path) {
                for (line_num, line) in content.lines().enumerate() {
                    if line.to_lowercase().contains(&ctx.keyword) {
                        let _ = result_tx.send(ScanResult {
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

        // EXIF data matching
        if ctx.scan_types.contains(&"exif_data".to_string()) && is_image_file(&extension) {
            if let Ok(exif_data) = extract_exif(&path) {
                if exif_data.to_lowercase().contains(&ctx.keyword) {
                    let _ = result_tx.send(ScanResult {
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

        // OCR text matching (macOS only)
        #[cfg(target_os = "macos")]
        if ctx.scan_types.contains(&"ocr_text".to_string()) && is_image_file(&extension) {
            if let Ok(ocr_text) = perform_ocr(&path) {
                if !ocr_text.is_empty() && ocr_text.to_lowercase().contains(&ctx.keyword) {
                    let _ = result_tx.send(ScanResult {
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

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn is_hidden(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    path_str.contains("/.") || path_str.contains("\\.") || file_name.starts_with('.')
}

fn matches_exclude(path: &Path, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let path_str = path.to_string_lossy().to_lowercase();
    let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
    patterns.iter().any(|p| {
        let p = p.to_lowercase();
        path_str.contains(&p) || file_name.contains(&p)
    })
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

fn count_entries_fast(path: &Path) -> u64 {
    fs::read_dir(path)
        .map(|entries| entries.filter_map(|e| e.ok()).count() as u64)
        .unwrap_or(0)
}

fn check_pause(pause_flag: &Arc<Mutex<bool>>, cancel_flag: &Arc<Mutex<bool>>) {
    while *pause_flag.lock().unwrap() {
        std::thread::sleep(Duration::from_millis(100));
        if *cancel_flag.lock().unwrap() {
            return;
        }
    }
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
