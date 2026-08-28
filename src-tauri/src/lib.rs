use crate::db::{Database, SettingsRepository};
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

mod clipboard;
mod commands;
mod db;
mod persian;
mod security;
mod window;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let database = Database::open(app.handle()).map_err(|error| error.to_user_message())?;
            let images_dir = app
                .path()
                .app_data_dir()
                .map_err(|_| "ذخیره‌سازی در دسترس نیست")?
                .join("images");
            clipboard::start(app.handle().clone(), database.clone(), images_dir);

            let launch_on_startup = database
                .read(|conn| SettingsRepository::new(conn).get())
                .map(|settings| settings.launch_on_startup)
                .unwrap_or(true);
            commands::apply_autostart(app.handle(), launch_on_startup);

            let _ = app.global_shortcut().on_shortcut(
                "CommandOrControl+Shift+V",
                |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        crate::window::toggle(app);
                    }
                },
            );

            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_clipboard_items,
            commands::search_clipboard_items,
            commands::pin_clipboard_item,
            commands::delete_clipboard_item,
            commands::clear_clipboard_history,
            commands::copy_clipboard_item,
            commands::get_settings,
            commands::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Clipoo");
}
