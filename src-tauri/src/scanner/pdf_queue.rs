use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use super::context::{ScanResult, extract_context};
use super::document::extract_document_text;

pub struct PdfTask {
    pub path: PathBuf,
    pub file_name: String,
    pub extension: String,
    pub file_size: u64,
}

pub struct PdfQueue {
    workers: Vec<JoinHandle<()>>,
}

impl PdfQueue {
    pub fn new(
        concurrent: usize,
        keyword: String,
        context_around: usize,
        result_tx: Sender<ScanResult>,
        cancel_flag: Arc<Mutex<bool>>,
    ) -> (Self, Sender<PdfTask>) {
        let (task_tx, task_rx) = std::sync::mpsc::channel::<PdfTask>();
        let task_rx = Arc::new(Mutex::new(task_rx));

        let mut workers = Vec::with_capacity(concurrent);
        for _ in 0..concurrent {
            let rx = task_rx.clone();
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
                            Self::process_task(&task, &kw, context_around, &rtx);
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
            });

            workers.push(handle);
        }

        (PdfQueue { workers }, task_tx)
    }

    pub fn wait_timeout(self, timeout: std::time::Duration) {
        let deadline = std::time::Instant::now() + timeout;
        let total = self.workers.len();
        for (i, handle) in self.workers.into_iter().enumerate() {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                log::warn!("PDF queue shutdown timed out after {}/{} workers, remaining threads detached", i, total);
                break;
            }
            match handle.join() {
                Ok(()) => {}
                Err(_) => log::warn!("PDF worker panicked"),
            }
        }
    }

    fn process_task(
        task: &PdfTask,
        keyword: &str,
        context_around: usize,
        result_tx: &Sender<ScanResult>,
    ) {
        match extract_document_text(&task.path, &task.extension) {
            Ok(content) => {
                if !content.is_empty() && content.to_lowercase().contains(keyword) {
                    let context_line = content
                        .lines()
                        .find(|line| line.to_lowercase().contains(keyword))
                        .unwrap_or("");
                    let _ = result_tx.send(ScanResult {
                        file_path: task.path.to_string_lossy().to_string(),
                        file_name: task.file_name.clone(),
                        match_type: "content".to_string(),
                        match_line: None,
                        match_context: Some(extract_context(
                            context_line,
                            keyword,
                            context_around,
                        )),
                        match_bboxes: None,
                        file_size: task.file_size,
                        file_extension: task.extension.clone(),
                        is_dir: false,
                    });
                }
            }
            Err(e) => {
                log::warn!("PDF extraction failed for {}: {}", task.path.display(), e);
            }
        }
    }
}
