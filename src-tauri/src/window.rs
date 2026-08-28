use tauri::{AppHandle, Emitter, Manager};

pub const WINDOW_SHOWN_EVENT: &str = "clipoo-window-shown";

pub fn show_centered(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit(WINDOW_SHOWN_EVENT, ());
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

pub fn toggle(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        if visible {
            hide(app);
        } else {
            show_centered(app);
        }
    }
}
