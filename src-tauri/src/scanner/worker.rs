use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};

use super::context::{ScanContext, ScanResult, extract_context};
use super::helpers::{is_hidden, matches_exclude, matches_rules, is_text_file, is_image_file, is_document_file};
use super::document::extract_document_text;
use super::matchers::extract_exif;
use crate::ocr::queue::OcrTask;

pub fn search_directory(
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

        if path.is_dir() {
            continue;
        }

        if is_hidden(&path) {
            continue;
        }

        if matches_exclude(&path, &ctx.exclude_patterns) {
            continue;
        }

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

        let count = files_scanned.fetch_add(1, Ordering::Relaxed) + 1;
        let _ = progress_tx.send((count, dir_str.clone()));

        let mut matched = false;

        // 1. OCR text matching — dispatch to async queue
        if ctx.scan_types.contains(&"ocr_text".to_string()) && is_image_file(&extension) {
            if let Some(ref ocr_tx) = ctx.ocr_queue {
                let _ = ocr_tx.send(OcrTask {
                    path: path.clone(),
                    file_name: file_name.clone(),
                    extension: extension.clone(),
                    file_size: metadata.len(),
                });
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

        // 4. Document content matching
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

        // 5. File name matching
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
