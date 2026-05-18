use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

use crate::scanner::ScanResult;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanConfig {
    pub path: String,
    pub keyword: String,
    pub scan_types: Vec<String>,
    pub file_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingConfirmation {
    pub id: String,
    pub path: String,
    pub entry_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkippedDir {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanProgress {
    pub scan_id: String,
    pub parent_scan_id: Option<String>,
    pub status: String,
    pub files_scanned: u32,
    pub results_found: u32,
    pub current_path: String,
    pub results: Vec<ScanResult>,
    pub pending_confirmations: Vec<PendingConfirmation>,
    pub skipped_dirs: Vec<SkippedDir>,
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

pub type ScanStore = Arc<Mutex<HashMap<String, ScanProgress>>>;

#[derive(Clone)]
pub struct CancelStore(pub Arc<Mutex<HashMap<String, Arc<Mutex<bool>>>>>);

pub struct DirWork {
    pub path: PathBuf,
}

pub type ChannelStore = Arc<Mutex<HashMap<String, Sender<DirWork>>>>;
