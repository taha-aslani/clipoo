use crate::clipboard::classify::classify_text;
use crate::db::{ClipboardItemType, NewClipboardItem};

pub enum RawCapture {
    Files(Vec<String>),
    Image(Vec<u8>),
    Text(String),
}

pub fn to_new_item(capture: RawCapture, image_path: Option<String>) -> Option<NewClipboardItem> {
    match capture {
        RawCapture::Files(paths) if !paths.is_empty() => {
            let preview = if paths.len() == 1 {
                std::path::Path::new(&paths[0])
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("فایل")
                    .to_string()
            } else {
                format!("{} فایل", paths.len())
            };

            Some(NewClipboardItem {
                item_type: ClipboardItemType::File,
                content: paths.join("\n"),
                preview: Some(preview),
            })
        }
        RawCapture::Files(_) => None,
        RawCapture::Image(_) => image_path.map(|content| NewClipboardItem {
            item_type: ClipboardItemType::Image,
            content,
            preview: Some("تصویر".to_string()),
        }),
        RawCapture::Text(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return None;
            }

            Some(NewClipboardItem {
                item_type: classify_text(trimmed),
                content: text,
                preview: None,
            })
        }
    }
}

#[cfg(windows)]
pub fn read_clipboard() -> Option<RawCapture> {
    use clipboard_win::formats::{Bitmap, FileList, Unicode};
    use clipboard_win::{get_clipboard, is_format_avail};

    if is_format_avail(clipboard_win::formats::CF_HDROP) {
        if let Ok(paths) = get_clipboard::<Vec<String>, _>(FileList) {
            if !paths.is_empty() {
                return Some(RawCapture::Files(paths));
            }
        }
    }

    if let Some(png) = read_png() {
        return Some(RawCapture::Image(png));
    }

    if is_format_avail(clipboard_win::formats::CF_DIB)
        || is_format_avail(clipboard_win::formats::CF_BITMAP)
    {
        if let Ok(bitmap) = get_clipboard::<Vec<u8>, _>(Bitmap) {
            if !bitmap.is_empty() {
                return Some(RawCapture::Image(bitmap));
            }
        }
    }

    if let Ok(text) = get_clipboard::<String, _>(Unicode) {
        if !text.trim().is_empty() {
            return Some(RawCapture::Text(text));
        }
    }

    None
}

#[cfg(windows)]
fn read_png() -> Option<Vec<u8>> {
    use clipboard_win::{is_format_avail, register_format, with_clipboard};

    let format = register_format("PNG")?;
    if !is_format_avail(format.get()) {
        return None;
    }

    let mut bytes = Vec::new();
    with_clipboard(|| {
        let _ = clipboard_win::raw::get(format.get(), &mut bytes);
    })
    .ok()?;

    if bytes.is_empty() {
        None
    } else {
        Some(bytes)
    }
}

#[cfg(not(windows))]
pub fn read_clipboard() -> Option<RawCapture> {
    None
}
