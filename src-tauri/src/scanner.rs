use rayon;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use std::time::Duration;
use uuid::Uuid;
use exif::Reader;

use crate::types::{PendingConfirmation, SkippedDir, ScanConfig, DirWork};
use crate::config::ContentExtractionSettings;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanResult {
    pub file_path: String,
    pub file_name: String,
    pub match_type: String,
    pub match_line: Option<u32>,
    pub match_context: Option<String>,
    pub match_bboxes: Option<String>,
    pub file_size: u64,
    pub file_extension: String,
    pub is_dir: bool,
}

#[derive(Debug, Deserialize)]
struct OCRRegion {
    text: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

pub struct ScanCallback {
    pub on_result: Box<dyn Fn(ScanResult) + Send>,
    pub on_progress: Box<dyn Fn(u32, String) + Send>,
    pub on_confirmation_needed: Box<dyn Fn(PendingConfirmation) + Send>,
    pub on_dir_skipped: Box<dyn Fn(SkippedDir) + Send>,
    pub should_cancel: Arc<Mutex<bool>>,
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
    context_around: usize,
    content_extraction: ContentExtractionSettings,
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
        context_around: app_config.display.match_context_length as usize,
        content_extraction: app_config.content_extraction.clone(),
    });

    let (result_tx, result_rx) = mpsc::channel::<ScanResult>();
    let (progress_tx, progress_rx) = mpsc::channel::<(u32, String)>();

    let cancel_flag = callback.should_cancel.clone();
    let files_scanned = Arc::new(AtomicU32::new(0));

    // 1. BFS thread: traverses tree, classifies directories
    let bfs_ctx = ctx.clone();
    let bfs_cancel = cancel_flag.clone();
    let bfs_work_tx = work_tx.clone();
    let bfs_handle = std::thread::spawn(move || {
        bfs_scan(
            &root,
            &bfs_ctx,
            bfs_work_tx,
            &*callback.on_confirmation_needed,
            &*callback.on_dir_skipped,
            &bfs_cancel,
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
    let rtx_main = result_tx.clone();
    let ptx_main = progress_tx.clone();

    let active_count = Arc::new(AtomicU32::new(0));
    let active_count_clone = active_count.clone();
    let bfs_done = Arc::new(AtomicBool::new(false));
    let bfs_done_clone = bfs_done.clone();

    let dispatch_handle = std::thread::spawn(move || {
        loop {
            // Try to receive work with a timeout
            match work_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(work) => {
                    if *cancel_flag.lock().unwrap() {
                        break;
                    }

                    let ctx = ctx.clone();
                    let rtx = rtx_main.clone();
                    let ptx = ptx_main.clone();
                    let fs = files_scanned.clone();
                    let cf = cancel_flag.clone();
                    let active = active_count_clone.clone();

                    active.fetch_add(1, Ordering::SeqCst);

                    rayon::spawn(move || {
                        search_directory(&work.path, &ctx, &rtx, &ptx, &fs, &cf);
                        active.fetch_sub(1, Ordering::SeqCst);
                    });
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // If BFS is done and no active tasks, we can stop
                    if bfs_done_clone.load(Ordering::SeqCst) && active_count_clone.load(Ordering::SeqCst) == 0 {
                        break;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // Channel closed, but wait for active tasks to finish
                    while active_count_clone.load(Ordering::SeqCst) > 0 {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    break;
                }
            }
        }
    });

    // Wait for BFS to finish
    let _ = bfs_handle.join();
    bfs_done.store(true, Ordering::SeqCst);
    let _ = dispatch_handle.join();

    // Give rayon tasks a moment to start
    std::thread::sleep(Duration::from_millis(50));

    // Wait for all rayon tasks to complete
    while active_count.load(Ordering::SeqCst) > 0 {
        std::thread::sleep(Duration::from_millis(10));
    }

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
) {
    let mut queue = VecDeque::new();
    queue.push_back(root.to_path_buf());

    // Track which directories have been sent to work_tx to avoid duplicates
    let mut sent_dirs: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    sent_dirs.insert(root.to_path_buf());
    let _ = work_tx.send(DirWork { path: root.to_path_buf() });

    while let Some(dir) = queue.pop_front() {
        if *cancel_flag.lock().unwrap() {
            break;
        }

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
                    enqueue_dir(&path, &ctx, &work_tx, on_confirmation_needed, on_dir_skipped, cancel_flag, &mut queue, &mut sent_dirs);
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
                enqueue_dir(&path, &ctx, &work_tx, on_confirmation_needed, on_dir_skipped, cancel_flag, &mut queue, &mut sent_dirs);
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
    _cancel_flag: &Arc<Mutex<bool>>,
    queue: &mut VecDeque<PathBuf>,
    sent_dirs: &mut std::collections::HashSet<PathBuf>,
) {
    // Skip if already sent to work queue
    if sent_dirs.contains(path) {
        return;
    }

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
        sent_dirs.insert(path.to_path_buf());
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
) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };

    let dir_str = dir.to_string_lossy().to_string();

    for entry in read_dir.filter_map(|e| e.ok()) {
        if *cancel_flag.lock().unwrap() {
            return;
        }

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

        // Try matching in priority order: OCR > EXIF > Content > Filename
        // Stop at first match to avoid duplicate results for the same file
        let mut matched = false;

        // 1. OCR text matching (highest priority for images)
        #[cfg(target_os = "macos")]
        if !matched && ctx.scan_types.contains(&"ocr_text".to_string()) && is_image_file(&extension) {
            if let Ok(regions) = perform_ocr(&path) {
                let all_text: Vec<&str> = regions.iter().map(|r| r.text.as_str()).collect();
                let joined = all_text.join("\n");
                if !joined.is_empty() && joined.to_lowercase().contains(&ctx.keyword) {
                    let matched_bboxes: Vec<serde_json::Value> = regions.iter()
                        .filter(|r| r.text.to_lowercase().contains(&ctx.keyword))
                        .map(|r| serde_json::json!({"x": r.x, "y": r.y, "w": r.w, "h": r.h}))
                        .collect();
                    let bboxes_json = if matched_bboxes.is_empty() {
                        None
                    } else {
                        Some(serde_json::to_string(&matched_bboxes).unwrap_or_default())
                    };
                    let _ = result_tx.send(ScanResult {
                        file_path: path.to_string_lossy().to_string(),
                        file_name: file_name.clone(),
                        match_type: "ocr".to_string(),
                        match_line: None,
                        match_context: Some(extract_context(&joined, &ctx.keyword, ctx.context_around)),
                        match_bboxes: bboxes_json,
                        file_size: metadata.len(),
                        file_extension: extension.clone(),
                        is_dir: false,
                    });
                    matched = true;
                }
            }
        }

        // 2. EXIF data matching
        if !matched && ctx.scan_types.contains(&"exif_data".to_string()) && is_image_file(&extension) {
            if let Ok(exif_data) = extract_exif(&path) {
                if exif_data.to_lowercase().contains(&ctx.keyword) {
                    let _ = result_tx.send(ScanResult {
                        file_path: path.to_string_lossy().to_string(),
                        file_name: file_name.clone(),
                        match_type: "exif".to_string(),
                        match_line: None,
                        match_context: Some(extract_context(&exif_data, &ctx.keyword, ctx.context_around)),
                        match_bboxes: None,
                        file_size: metadata.len(),
                        file_extension: extension.clone(),
                        is_dir: false,
                    });
                    matched = true;
                }
            }
        }

        // 3. Text content matching
        if !matched && ext_allowed && ctx.scan_types.contains(&"text_content".to_string()) && is_text_file(&extension) {
            if let Ok(content) = fs::read_to_string(&path) {
                for (line_num, line) in content.lines().enumerate() {
                    if line.to_lowercase().contains(&ctx.keyword) {
                        let _ = result_tx.send(ScanResult {
                            file_path: path.to_string_lossy().to_string(),
                            file_name: file_name.clone(),
                        match_type: "content".to_string(),
                        match_line: Some((line_num + 1) as u32),
                        match_context: Some(extract_context(line, &ctx.keyword, ctx.context_around)),
                        match_bboxes: None,
                        file_size: metadata.len(),
                            file_extension: extension.clone(),
                            is_dir: false,
                        });
                        matched = true;
                        break;
                    }
                }
            }
        }

        // 3.5. Document content matching (docx, xlsx, pptx, pdf)
        if !matched && ext_allowed && ctx.scan_types.contains(&"document_content".to_string()) && is_document_file(&extension) {
            let is_enabled = match extension.as_str() {
                "docx" => ctx.content_extraction.docx,
                "xlsx" => ctx.content_extraction.xlsx,
                "pdf" => ctx.content_extraction.pdf,
                "pptx" => ctx.content_extraction.pptx,
                _ => false,
            };
            if is_enabled {
                if let Ok(content) = extract_document_text(&path, &extension) {
                    let content_lower = content.to_lowercase();
                    if content_lower.contains(&ctx.keyword) {
                        let context_line = content.lines()
                            .find(|line| line.to_lowercase().contains(&ctx.keyword))
                            .unwrap_or("");
                        let _ = result_tx.send(ScanResult {
                            file_path: path.to_string_lossy().to_string(),
                            file_name: file_name.clone(),
                            match_type: "content".to_string(),
                            match_line: None,
                            match_context: Some(extract_context(context_line, &ctx.keyword, ctx.context_around)),
                            match_bboxes: None,
                            file_size: metadata.len(),
                            file_extension: extension.clone(),
                            is_dir: false,
                        });
                        matched = true;
                    }
                }
            }
        }

        // 4. File name matching (lowest priority)
        if !matched && ext_allowed && ctx.scan_types.contains(&"file_name".to_string()) {
            if file_name.to_lowercase().contains(&ctx.keyword) {
                let _ = result_tx.send(ScanResult {
                    file_path: path.to_string_lossy().to_string(),
                    file_name: file_name.clone(),
                        match_type: "filename".to_string(),
                        match_line: None,
                        match_context: Some(extract_context(&file_name, &ctx.keyword, ctx.context_around)),
                        match_bboxes: None,
                        file_size: metadata.len(),
                    file_extension: extension.clone(),
                    is_dir: false,
                });
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

pub fn is_text_file(extension: &str) -> bool {
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

pub fn is_document_file(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "docx" | "xlsx" | "pptx" | "pdf"
    )
}

fn extract_docx_text(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut doc = archive.by_name("word/document.xml").map_err(|e| e.to_string())?;
    let mut contents = String::new();
    use std::io::Read;
    doc.read_to_string(&mut contents).map_err(|e| e.to_string())?;
    let mut text = String::new();
    let mut reader = quick_xml::Reader::from_str(&contents);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(ref e)) | Ok(quick_xml::events::Event::Empty(ref e)) => {
                if e.local_name().as_ref() == b"t" {
                    if let Ok(quick_xml::events::Event::Text(t)) = reader.read_event_into(&mut Vec::new()) {
                        let s = t.unescape().unwrap_or_default();
                        if !s.is_empty() {
                            text.push_str(&s);
                            text.push('\n');
                        }
                    }
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(text)
}

fn extract_xlsx_text(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut shared_strings = Vec::new();
    if let Ok(mut ss_file) = archive.by_name("xl/sharedStrings.xml") {
        let mut contents = String::new();
        use std::io::Read;
        ss_file.read_to_string(&mut contents).map_err(|e| e.to_string())?;
        let mut reader = quick_xml::Reader::from_str(&contents);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) | Ok(quick_xml::events::Event::Empty(ref e)) => {
                    if e.local_name().as_ref() == b"t" {
                        if let Ok(quick_xml::events::Event::Text(t)) = reader.read_event_into(&mut Vec::new()) {
                            shared_strings.push(t.unescape().unwrap_or_default().to_string());
                        }
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
    }
    let mut sheet_names: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(name) = archive.by_index(i) {
            let n = name.name().to_string();
            if n.starts_with("xl/worksheets/sheet") && n.ends_with(".xml") {
                sheet_names.push(n);
            }
        }
    }
    let mut text = String::new();
    for sheet_name in sheet_names {
        if let Ok(mut sheet_file) = archive.by_name(&sheet_name) {
            let mut contents = String::new();
            use std::io::Read;
            sheet_file.read_to_string(&mut contents).map_err(|e| e.to_string())?;
            let mut reader = quick_xml::Reader::from_str(&contents);
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(quick_xml::events::Event::Start(ref e)) | Ok(quick_xml::events::Event::Empty(ref e)) => {
                        if e.local_name().as_ref() == b"v" {
                            if let Ok(quick_xml::events::Event::Text(t)) = reader.read_event_into(&mut Vec::new()) {
                                let val = t.unescape().unwrap_or_default().to_string();
                                if let Ok(idx) = val.parse::<usize>() {
                                    if let Some(s) = shared_strings.get(idx) {
                                        text.push_str(s);
                                        text.push('\n');
                                    }
                                } else {
                                    text.push_str(&val);
                                    text.push('\n');
                                }
                            }
                        }
                    }
                    Ok(quick_xml::events::Event::Eof) => break,
                    Err(_) => break,
                    _ => {}
                }
                buf.clear();
            }
        }
    }
    Ok(text)
}

fn extract_pptx_text(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut slide_names: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let n = entry.name().to_string();
            if n.starts_with("ppt/slides/slide") && n.ends_with(".xml") {
                slide_names.push(n);
            }
        }
    }
    let mut text = String::new();
    for slide_name in slide_names {
        if let Ok(mut slide_file) = archive.by_name(&slide_name) {
            let mut contents = String::new();
            use std::io::Read;
            slide_file.read_to_string(&mut contents).map_err(|e| e.to_string())?;
            let mut reader = quick_xml::Reader::from_str(&contents);
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(quick_xml::events::Event::Start(ref e)) | Ok(quick_xml::events::Event::Empty(ref e)) => {
                        if e.local_name().as_ref() == b"t" {
                            if let Ok(quick_xml::events::Event::Text(t)) = reader.read_event_into(&mut Vec::new()) {
                                let s = t.unescape().unwrap_or_default();
                                if !s.is_empty() {
                                    text.push_str(&s);
                                    text.push('\n');
                                }
                            }
                        }
                    }
                    Ok(quick_xml::events::Event::Eof) => break,
                    Err(_) => break,
                    _ => {}
                }
                buf.clear();
            }
        }
    }
    Ok(text)
}

fn extract_pdf_text(path: &Path) -> Result<String, String> {
    let path = path.to_path_buf();
    std::panic::catch_unwind(move || {
        pdf_extract::extract_text(&path)
    })
    .map_err(|_| "PDF extraction panicked (unsupported encoding)".to_string())?
    .map_err(|e| e.to_string())
}

pub fn extract_document_text(path: &Path, extension: &str) -> Result<String, String> {
    match extension {
        "docx" => extract_docx_text(path),
        "xlsx" => extract_xlsx_text(path),
        "pptx" => extract_pptx_text(path),
        "pdf" => extract_pdf_text(path),
        _ => Err("Unsupported document type".to_string()),
    }
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

fn perform_ocr(path: &Path) -> Result<Vec<OCRRegion>, String> {
    let script = include_str!("../resources/ocr.swift");
    let temp_dir = std::env::temp_dir();
    let temp_script = temp_dir.join("lumina_ocr.swift");
    fs::write(&temp_script, script).map_err(|e| e.to_string())?;
    let output = Command::new("swift")
        .arg(&temp_script)
        .arg(path.to_string_lossy().to_string())
        .output()
        .map_err(|e| e.to_string())?;
    let _ = fs::remove_file(&temp_script);
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let trimmed = stdout.trim().to_string();
        if trimmed.starts_with('[') {
            serde_json::from_str(&trimmed).map_err(|e| e.to_string())
        } else if trimmed.starts_with("ERROR") {
            Err(trimmed)
        } else {
            Err("Unexpected OCR output".to_string())
        }
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        Err(error)
    }
}
