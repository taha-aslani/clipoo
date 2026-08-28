use std::path::{Path, PathBuf};

use crate::db::{ClipboardItem, ClipboardItemType};

pub fn save_png(images_dir: &Path, bytes: &[u8]) -> Result<PathBuf, ()> {
    std::fs::create_dir_all(images_dir).map_err(|_| ())?;
    let file_name = format!("{}.png", uuid::Uuid::new_v4());
    let path = images_dir.join(file_name);
    let image = image::load_from_memory(bytes).map_err(|_| ())?;
    image
        .save_with_format(&path, image::ImageFormat::Png)
        .map_err(|_| ())?;
    Ok(path)
}

pub fn delete_if_image(item: &ClipboardItem) {
    if item.item_type != ClipboardItemType::Image {
        return;
    }

    let path = Path::new(&item.content);
    if path.is_file() {
        let _ = std::fs::remove_file(path);
    }
}
