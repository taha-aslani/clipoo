use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::clipboard::capture::{read_clipboard, to_new_item, RawCapture};
use crate::clipboard::images::save_png;
use crate::db::{Database, SaveOutcome, SettingsRepository};
use crate::security::is_excluded_foreground_app;

const POLL_INTERVAL: Duration = Duration::from_millis(100);
pub const ITEM_ADDED_EVENT: &str = "clipboard-item-added";

pub fn spawn(app: AppHandle, db: Database, images_dir: PathBuf) {
    let _ = thread::Builder::new()
        .name("clipoo-clipboard".into())
        .spawn(move || run(app, db, images_dir));
}

fn run(app: AppHandle, db: Database, images_dir: PathBuf) {
    let mut last_sequence = current_sequence();
    let mut last_image_hash: Option<u64> = None;

    loop {
        thread::sleep(POLL_INTERVAL);

        let sequence = current_sequence();
        if sequence == last_sequence {
            continue;
        }
        last_sequence = sequence;

        if crate::clipboard::take_skipped_sequence(sequence)
            || crate::clipboard::take_suppress_next_capture()
        {
            continue;
        }

        if !monitoring_enabled(&db) {
            last_image_hash = None;
            continue;
        }

        if is_excluded_foreground_app() {
            continue;
        }

        let Some(capture) = read_clipboard() else {
            continue;
        };

        if let RawCapture::Image(bytes) = &capture {
            let hash = hash_bytes(bytes);
            if last_image_hash == Some(hash) {
                continue;
            }
            last_image_hash = Some(hash);
        } else {
            last_image_hash = None;
        }

        let image_path = match &capture {
            RawCapture::Image(bytes) => match save_png(&images_dir, bytes) {
                Ok(path) => Some(path.to_string_lossy().into_owned()),
                Err(()) => continue,
            },
            _ => None,
        };

        let Some(new_item) = to_new_item(capture, image_path.clone()) else {
            if let Some(path) = image_path {
                let _ = std::fs::remove_file(path);
            }
            continue;
        };

        match db.save_clipboard_item(new_item) {
            Ok(SaveOutcome::Inserted(item)) => {
                let _ = app.emit(ITEM_ADDED_EVENT, &item);
            }
            Ok(SaveOutcome::Duplicate) => {
                if let Some(path) = image_path {
                    let _ = std::fs::remove_file(path);
                }
            }
            Err(_) => {
                if let Some(path) = image_path {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
    }
}

fn monitoring_enabled(db: &Database) -> bool {
    db.read(|conn| SettingsRepository::new(conn).get())
        .map(|settings| settings.enable_monitoring)
        .unwrap_or(true)
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

#[cfg(windows)]
fn current_sequence() -> u32 {
    clipboard_win::seq_num().map(|value| value.get()).unwrap_or(0)
}

#[cfg(not(windows))]
fn current_sequence() -> u32 {
    0
}
