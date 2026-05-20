use serde::{Deserialize, Serialize};

use crate::config::ContentExtractionSettings;
use crate::ocr::OcrProvider;
use std::sync::{Arc, Mutex};

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

pub struct ScanCallback {
    pub on_result: Box<dyn Fn(ScanResult) + Send>,
    pub on_progress: Box<dyn Fn(u32, String) + Send>,
    pub on_confirmation_needed: Box<dyn Fn(crate::types::PendingConfirmation) + Send>,
    pub on_dir_skipped: Box<dyn Fn(crate::types::SkippedDir) + Send>,
    pub should_cancel: Arc<Mutex<bool>>,
}

#[derive(Clone)]
pub struct ScanContext {
    pub keyword: String,
    pub scan_types: Vec<String>,
    pub file_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub skip_rules: Vec<String>,
    pub scan_rules: Vec<String>,
    pub threshold: u64,
    pub ask_on_large_dir: bool,
    pub context_around: usize,
    pub content_extraction: ContentExtractionSettings,
    pub ocr_provider: Option<Arc<dyn OcrProvider>>,
}

pub fn extract_context(line: &str, keyword: &str, context_around: usize) -> String {
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
