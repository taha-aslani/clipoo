use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

use crate::clipboard::{delete_if_image, write_item};
use crate::db::{
    AppSettings, ClipboardFilter, ClipboardItem, ClipboardRepository, Database, SettingsRepository,
};

#[tauri::command]
pub fn list_clipboard_items(
    db: State<'_, Database>,
    filter: ClipboardFilter,
    limit: Option<u32>,
) -> Result<Vec<ClipboardItem>, String> {
    db.read(|conn| ClipboardRepository::new(conn).list(filter, limit))
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn search_clipboard_items(
    db: State<'_, Database>,
    query: String,
    filter: ClipboardFilter,
    limit: Option<u32>,
) -> Result<Vec<ClipboardItem>, String> {
    db.read(|conn| ClipboardRepository::new(conn).search(&query, filter, limit))
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn pin_clipboard_item(
    db: State<'_, Database>,
    id: String,
    pinned: bool,
) -> Result<ClipboardItem, String> {
    db.write(|conn| ClipboardRepository::new(conn).set_pinned(&id, pinned))
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn delete_clipboard_item(db: State<'_, Database>, id: String) -> Result<(), String> {
    let item = db
        .write(|conn| {
            let repo = ClipboardRepository::new(conn);
            let item = repo.get(&id)?;
            repo.delete(&id)?;
            Ok(item)
        })
        .map_err(|error| error.to_user_message())?;

    delete_if_image(&item);
    Ok(())
}

#[tauri::command]
pub fn clear_clipboard_history(db: State<'_, Database>) -> Result<usize, String> {
    let images = db
        .read(|conn| ClipboardRepository::new(conn).list(ClipboardFilter::Image, Some(u32::MAX)))
        .map_err(|error| error.to_user_message())?;

    let deleted = db
        .write(|conn| ClipboardRepository::new(conn).clear())
        .map_err(|error| error.to_user_message())?;

    for item in images {
        delete_if_image(&item);
    }

    Ok(deleted)
}

#[tauri::command]
pub fn copy_clipboard_item(db: State<'_, Database>, id: String) -> Result<(), String> {
    let item = db
        .read(|conn| ClipboardRepository::new(conn).get(&id))
        .map_err(|error| error.to_user_message())?;

    write_item(&item).map_err(|_| "کپی انجام نشد".to_string())
}

#[tauri::command]
pub fn get_settings(db: State<'_, Database>) -> Result<AppSettings, String> {
    db.read(|conn| SettingsRepository::new(conn).get())
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    db: State<'_, Database>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let settings = db
        .write(|conn| {
            let settings = SettingsRepository::new(conn).update(&settings)?;
            let evicted = ClipboardRepository::new(conn).enforce_max_history(settings.max_history_size)?;
            Ok((settings, evicted))
        })
        .map_err(|error| error.to_user_message())?;

    let (settings, evicted) = settings;
    for item in evicted {
        delete_if_image(&item);
    }

    apply_autostart(&app, settings.launch_on_startup);
    Ok(settings)
}

pub(crate) fn apply_autostart(app: &AppHandle, enabled: bool) {
    if enabled {
        let _ = app.autolaunch().enable();
    } else {
        let _ = app.autolaunch().disable();
    }
}
