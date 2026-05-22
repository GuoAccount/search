mod bfs;
mod context;
mod document;
mod helpers;
mod matchers;
mod pdf_queue;
mod worker;

pub use context::{ScanResult, ScanCallback, ScanContext, extract_context};
pub use document::extract_document_text;
pub use helpers::{is_text_file, is_document_file};

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use std::time::Duration;

use crate::types::DirWork;
use crate::config::AppConfig;
use crate::ocr;
use crate::ocr::queue::OcrQueue;
use pdf_queue::PdfQueue;

pub fn scan_directory(
    config: crate::types::ScanConfig,
    app_config: AppConfig,
    callback: ScanCallback,
    work_tx: Sender<DirWork>,
    work_rx: Receiver<DirWork>,
) {
    let root = PathBuf::from(&config.path);
    
    let ocr_provider: Option<Arc<dyn crate::ocr::OcrProvider>> = if app_config.ocr.enabled && config.scan_types.contains(&"ocr_text".to_string()) {
        let provider = ocr::create_ocr_provider(&app_config.ocr);
        if provider.is_available() {
            Some(Arc::from(provider))
        } else {
            log::warn!("OCR provider '{}' is not available", provider.name());
            None
        }
    } else {
        None
    };

    let (result_tx, result_rx) = mpsc::channel::<ScanResult>();
    let (progress_tx, progress_rx) = mpsc::channel::<(u32, String)>();

    let cancel_flag = callback.should_cancel.clone();
    let files_scanned = Arc::new(AtomicU32::new(0));

    let (ocr_queue_handle, ocr_task_tx) = if let Some(ref provider) = ocr_provider {
        let concurrent = app_config.ocr.concurrent.max(1);
        let keyword = config.keyword.to_lowercase();
        let context_around = app_config.display.match_context_length as usize;
        let (queue, tx) = OcrQueue::new(
            concurrent,
            provider.clone(),
            keyword,
            context_around,
            result_tx.clone(),
            cancel_flag.clone(),
        );
        (Some(queue), Some(tx))
    } else {
        (None, None)
    };

    // PDF queue: limit concurrent PDF processing
    let pdf_concurrent = 2; // Limit to 2 concurrent PDF extractions
    let pdf_keyword = config.keyword.to_lowercase();
    let pdf_context_around = app_config.display.match_context_length as usize;
    let (pdf_queue_handle, pdf_task_tx) = PdfQueue::new(
        pdf_concurrent,
        pdf_keyword,
        pdf_context_around,
        result_tx.clone(),
        cancel_flag.clone(),
    );

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
        ocr_queue: ocr_task_tx.clone(),
        pdf_queue: Some(pdf_task_tx.clone()),
    });

    let bfs_ctx = ctx.clone();
    let bfs_cancel = cancel_flag.clone();
    let bfs_work_tx = work_tx.clone();
    let bfs_handle = std::thread::spawn(move || {
        bfs::bfs_scan(
            &root,
            &bfs_ctx,
            bfs_work_tx,
            &*callback.on_confirmation_needed,
            &*callback.on_dir_skipped,
            &bfs_cancel,
        );
    });

    let result_handle = std::thread::spawn(move || {
        for result in result_rx {
            (callback.on_result)(result);
        }
    });

    let progress_handle = std::thread::spawn(move || {
        for (count, path) in progress_rx {
            (callback.on_progress)(count, path);
        }
    });

    let rtx_main = result_tx.clone();
    let ptx_main = progress_tx.clone();

    let active_count = Arc::new(AtomicU32::new(0));
    let active_count_clone = active_count.clone();
    let bfs_done = Arc::new(AtomicBool::new(false));
    let bfs_done_clone = bfs_done.clone();

    let dispatch_handle = std::thread::spawn(move || {
        loop {
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
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            worker::search_directory(&work.path, &ctx, &rtx, &ptx, &fs, &cf);
                        }));
                        if let Err(e) = result {
                            log::error!("search_directory panicked for {}: {:?}", work.path.display(), e);
                        }
                        active.fetch_sub(1, Ordering::SeqCst);
                    });
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if bfs_done_clone.load(Ordering::SeqCst) && active_count_clone.load(Ordering::SeqCst) == 0 {
                        break;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    while active_count_clone.load(Ordering::SeqCst) > 0 {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    break;
                }
            }
        }
    });

    let _ = bfs_handle.join();
    bfs_done.store(true, Ordering::SeqCst);
    let _ = dispatch_handle.join();

    std::thread::sleep(Duration::from_millis(50));

    while active_count.load(Ordering::SeqCst) > 0 {
        std::thread::sleep(Duration::from_millis(10));
    }

    drop(ocr_task_tx);
    if let Some(queue) = ocr_queue_handle {
        queue.wait_timeout(Duration::from_secs(30));
    }

    drop(pdf_task_tx);
    pdf_queue_handle.wait_timeout(Duration::from_secs(30));

    drop(result_tx);
    drop(progress_tx);
    let _ = result_handle.join();
    let _ = progress_handle.join();
}
