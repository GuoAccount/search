use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::scanner::{extract_context, ScanResult};
use super::OcrProvider;

pub struct OcrTask {
    pub path: PathBuf,
    pub file_name: String,
    pub extension: String,
    pub file_size: u64,
}

pub struct OcrQueue {
    workers: Vec<JoinHandle<()>>,
}

impl OcrQueue {
    pub fn new(
        concurrent: usize,
        ocr_provider: Arc<dyn OcrProvider>,
        keyword: String,
        context_around: usize,
        result_tx: Sender<ScanResult>,
        cancel_flag: Arc<Mutex<bool>>,
    ) -> (Self, Sender<OcrTask>) {
        let (task_tx, task_rx) = std::sync::mpsc::channel::<OcrTask>();
        let task_rx = Arc::new(Mutex::new(task_rx));

        let mut workers = Vec::with_capacity(concurrent);
        for _ in 0..concurrent {
            let rx = task_rx.clone();
            let provider = ocr_provider.clone();
            let kw = keyword.clone();
            let rtx = result_tx.clone();
            let cf = cancel_flag.clone();

            let handle = thread::spawn(move || {
                loop {
                    if *cf.lock().unwrap() {
                        break;
                    }

                    let task = {
                        let guard = rx.lock().unwrap();
                        guard.recv_timeout(std::time::Duration::from_millis(100))
                    };

                    match task {
                        Ok(task) => {
                            Self::process_task(&*provider, &task, &kw, context_around, &rtx);
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
            });

            workers.push(handle);
        }

        (OcrQueue { workers }, task_tx)
    }

    pub fn wait_timeout(self, timeout: std::time::Duration) {
        let deadline = std::time::Instant::now() + timeout;
        let total = self.workers.len();
        for (i, handle) in self.workers.into_iter().enumerate() {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                log::warn!("OCR queue shutdown timed out after {}/{} workers, remaining threads detached", i, total);
                break;
            }
            match handle.join() {
                Ok(()) => {}
                Err(_) => log::warn!("OCR worker panicked"),
            }
        }
    }

    fn process_task(
        provider: &dyn OcrProvider,
        task: &OcrTask,
        keyword: &str,
        context_around: usize,
        result_tx: &Sender<ScanResult>,
    ) {
        match provider.recognize(&task.path) {
            Ok(result) => {
                if !result.raw_text.is_empty()
                    && result.raw_text.to_lowercase().contains(keyword)
                {
                    let matched_bboxes: Vec<serde_json::Value> = result
                        .regions
                        .iter()
                        .filter(|r| r.text.to_lowercase().contains(keyword))
                        .map(|r| {
                            serde_json::json!({"x": r.x, "y": r.y, "w": r.w, "h": r.h})
                        })
                        .collect();

                    let bboxes_json = if matched_bboxes.is_empty() {
                        None
                    } else {
                        Some(serde_json::to_string(&matched_bboxes).unwrap_or_default())
                    };

                    let _ = result_tx.send(ScanResult {
                        file_path: task.path.to_string_lossy().to_string(),
                        file_name: task.file_name.clone(),
                        match_type: "ocr".to_string(),
                        match_line: None,
                        match_context: Some(extract_context(
                            &result.raw_text,
                            keyword,
                            context_around,
                        )),
                        match_bboxes: bboxes_json,
                        file_size: task.file_size,
                        file_extension: task.extension.clone(),
                        is_dir: false,
                    });
                }
            }
            Err(e) => {
                log::warn!("OCR failed for {}: {}", task.path.display(), e);
            }
        }
    }
}
