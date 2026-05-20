use std::collections::{VecDeque, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use uuid::Uuid;

use crate::types::{PendingConfirmation, SkippedDir, DirWork};
use super::context::ScanContext;
use super::helpers::{is_hidden, matches_exclude, matches_rules, count_entries_fast};

pub fn bfs_scan(
    root: &Path,
    ctx: &ScanContext,
    work_tx: Sender<DirWork>,
    on_confirmation_needed: &dyn Fn(PendingConfirmation),
    on_dir_skipped: &dyn Fn(SkippedDir),
    cancel_flag: &Arc<Mutex<bool>>,
) {
    let mut queue = VecDeque::new();
    queue.push_back(root.to_path_buf());

    let mut sent_dirs: HashSet<PathBuf> = HashSet::new();
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

            if is_hidden(&path) {
                continue;
            }

            if matches_exclude(&path, &ctx.exclude_patterns) {
                continue;
            }

            if matches_rules(&path, &ctx.scan_rules) {
                if path.is_dir() {
                    enqueue_dir(&path, ctx, &work_tx, on_confirmation_needed, on_dir_skipped, &mut queue, &mut sent_dirs, true);
                }
                continue;
            }

            if matches_rules(&path, &ctx.skip_rules) {
                if path.is_dir() {
                    on_dir_skipped(SkippedDir {
                        path: path.to_string_lossy().to_string(),
                        reason: "rule".to_string(),
                    });
                }
                continue;
            }

            if path.is_dir() {
                enqueue_dir(&path, ctx, &work_tx, on_confirmation_needed, on_dir_skipped, &mut queue, &mut sent_dirs, false);
            }
        }
    }
}

fn enqueue_dir(
    path: &Path,
    ctx: &ScanContext,
    work_tx: &Sender<DirWork>,
    on_confirmation_needed: &dyn Fn(PendingConfirmation),
    on_dir_skipped: &dyn Fn(SkippedDir),
    queue: &mut VecDeque<PathBuf>,
    sent_dirs: &mut HashSet<PathBuf>,
    force: bool,
) {
    if sent_dirs.contains(path) {
        return;
    }

    let count = count_entries_fast(path);

    if !force && count > ctx.threshold && ctx.ask_on_large_dir {
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
        sent_dirs.insert(path.to_path_buf());
        let _ = work_tx.send(DirWork { path: path.to_path_buf() });
        queue.push_back(path.to_path_buf());
    }
}
