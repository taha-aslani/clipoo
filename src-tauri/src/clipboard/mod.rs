mod capture;
mod classify;
mod images;
mod monitor;
mod suppress;
mod write;

pub use images::delete_if_image;
pub use monitor::spawn;
pub use suppress::{take_skipped_sequence, take_suppress_next_capture};
pub use write::write_item;

use std::path::PathBuf;

use tauri::AppHandle;

use crate::db::Database;

pub fn start(app: AppHandle, db: Database, images_dir: PathBuf) {
    let _ = std::fs::create_dir_all(&images_dir);
    spawn(app, db, images_dir);
}
