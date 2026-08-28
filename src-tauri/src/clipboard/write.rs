use crate::clipboard::suppress::suppress_next_capture;
use crate::db::{ClipboardItem, ClipboardItemType};

pub fn write_item(item: &ClipboardItem) -> Result<(), ()> {
    suppress_next_capture();

    let result = write_payload(item);
    if result.is_err() {
        let _ = crate::clipboard::suppress::take_suppress_next_capture();
    } else {
        #[cfg(windows)]
        if let Some(sequence) = clipboard_win::seq_num() {
            crate::clipboard::suppress::skip_clipboard_sequence(sequence.get());
        }
    }
    result
}

fn write_payload(item: &ClipboardItem) -> Result<(), ()> {
    match item.item_type {
        ClipboardItemType::File => write_files(&item.content),
        ClipboardItemType::Image => write_image(&item.content),
        ClipboardItemType::Text | ClipboardItemType::Url | ClipboardItemType::Code => {
            write_text(&item.content)
        }
    }
}

#[cfg(windows)]
fn write_text(content: &str) -> Result<(), ()> {
    clipboard_win::set_clipboard_string(content).map_err(|_| ())
}

#[cfg(windows)]
fn write_files(content: &str) -> Result<(), ()> {
    use clipboard_win::formats::FileList;
    use clipboard_win::{with_clipboard, Setter};

    let paths: Vec<String> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();

    if paths.is_empty() {
        return Err(());
    }

    let mut failed = false;
    with_clipboard(|| {
        if clipboard_win::empty().is_err() || FileList.write_clipboard(paths.as_slice()).is_err() {
            failed = true;
        }
    })
    .map_err(|_| ())?;

    if failed { Err(()) } else { Ok(()) }
}

#[cfg(windows)]
fn write_image(path: &str) -> Result<(), ()> {
    use clipboard_win::raw;
    use clipboard_win::{empty, register_format, with_clipboard};

    let bytes = std::fs::read(path).map_err(|_| ())?;
    if bytes.is_empty() {
        return Err(());
    }

    let format = register_format("PNG").ok_or(())?;
    let mut failed = false;
    with_clipboard(|| {
        if empty().is_err() || raw::set(format.get(), &bytes).is_err() {
            failed = true;
        }
    })
    .map_err(|_| ())?;

    if failed { Err(()) } else { Ok(()) }
}

#[cfg(not(windows))]
fn write_text(_content: &str) -> Result<(), ()> {
    Err(())
}

#[cfg(not(windows))]
fn write_files(_content: &str) -> Result<(), ()> {
    Err(())
}

#[cfg(not(windows))]
fn write_image(_path: &str) -> Result<(), ()> {
    Err(())
}
