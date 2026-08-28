mod error;
mod fts;
mod migrate;
mod models;
pub mod repositories;

pub use error::{DbError, DbResult};
pub use models::{
    AppSettings, ClipboardFilter, ClipboardItem, ClipboardItemType, NewClipboardItem, SaveOutcome,
};
pub use repositories::{ClipboardRepository, SettingsRepository};

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(app: &AppHandle) -> DbResult<Self> {
        let dir = app.path().app_data_dir().map_err(|_| DbError::Unavailable)?;
        std::fs::create_dir_all(&dir).map_err(|_| DbError::Unavailable)?;
        Self::open_path(&dir.join("clipoo.db"))
    }

    pub fn open_path(path: &Path) -> DbResult<Self> {
        let conn = Connection::open(path)?;
        configure(&conn, true)?;
        migrate::apply(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    #[cfg(test)]
    pub fn open_in_memory() -> DbResult<Self> {
        let conn = Connection::open_in_memory()?;
        configure(&conn, false)?;
        migrate::apply(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn read<T>(&self, f: impl FnOnce(&Connection) -> DbResult<T>) -> DbResult<T> {
        let conn = self.conn.lock().map_err(|_| DbError::Poisoned)?;
        f(&conn)
    }

    pub fn write<T>(&self, f: impl FnOnce(&Connection) -> DbResult<T>) -> DbResult<T> {
        let mut conn = self.conn.lock().map_err(|_| DbError::Poisoned)?;
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }

    pub fn save_clipboard_item(&self, new_item: NewClipboardItem) -> DbResult<SaveOutcome> {
        let (outcome, evicted) = self.write(|conn| {
            let clipboard = ClipboardRepository::new(conn);
            let outcome = clipboard.insert(new_item)?;
            let evicted = if matches!(outcome, SaveOutcome::Inserted(_)) {
                let settings = SettingsRepository::new(conn).get()?;
                clipboard.enforce_max_history(settings.max_history_size)?
            } else {
                Vec::new()
            };
            Ok((outcome, evicted))
        })?;

        for item in evicted {
            remove_stored_image(&item);
        }

        Ok(outcome)
    }
}

fn remove_stored_image(item: &ClipboardItem) {
    if item.item_type != ClipboardItemType::Image {
        return;
    }

    let path = Path::new(&item.content);
    if path.is_file() {
        let _ = std::fs::remove_file(path);
    }
}

fn configure(conn: &Connection, persistent: bool) -> rusqlite::Result<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "busy_timeout", "5000")?;
    conn.pragma_update(None, "secure_delete", "ON")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;

    if persistent {
        let _: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{ClipboardItemType, NewClipboardItem};
    use crate::persian::normalize_persian;

    fn save_text(db: &Database, content: &str) -> SaveOutcome {
        db.write(|conn| {
            ClipboardRepository::new(conn).insert(NewClipboardItem {
                item_type: ClipboardItemType::Text,
                content: content.to_string(),
                preview: None,
            })
        })
        .expect("insert should succeed")
    }

    #[test]
    fn persian_variants_share_one_search_result_set() {
        let db = Database::open_in_memory().expect("memory db");
        assert!(matches!(save_text(&db, "کتاب"), SaveOutcome::Inserted(_)));
        assert!(matches!(save_text(&db, "مسئول"), SaveOutcome::Inserted(_)));
        assert!(matches!(save_text(&db, "می"), SaveOutcome::Inserted(_)));

        let searches = ["كتاب", "مسول", "مي"];
        for query in searches {
            let results = db
                .read(|conn| {
                    ClipboardRepository::new(conn).search(query, ClipboardFilter::All, None)
                })
                .expect("search should succeed");
            assert_eq!(results.len(), 1, "query {query} should match exactly one item");
        }
    }

    #[test]
    fn zwnj_and_spaced_phrases_match() {
        let db = Database::open_in_memory().expect("memory db");
        assert!(matches!(
            save_text(&db, "خانه من"),
            SaveOutcome::Inserted(_)
        ));

        let results = db
            .read(|conn| {
                ClipboardRepository::new(conn).search("خانه‌من", ClipboardFilter::All, None)
            })
            .expect("search should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "خانه من");
        assert_eq!(
            results[0].normalized_content,
            normalize_persian("خانه‌من")
        );
    }

    #[test]
    fn consecutive_duplicates_use_normalized_text() {
        let db = Database::open_in_memory().expect("memory db");
        assert!(matches!(save_text(&db, "کتاب"), SaveOutcome::Inserted(_)));
        assert!(matches!(save_text(&db, "كتاب"), SaveOutcome::Duplicate));
        let count = db
            .read(|conn| ClipboardRepository::new(conn).count())
            .expect("count");
        assert_eq!(count, 1);
    }

    #[test]
    fn clear_permanently_deletes_rows() {
        let db = Database::open_in_memory().expect("memory db");
        assert!(matches!(save_text(&db, "یک"), SaveOutcome::Inserted(_)));
        assert!(matches!(save_text(&db, "دو"), SaveOutcome::Inserted(_)));

        let deleted = db
            .write(|conn| ClipboardRepository::new(conn).clear())
            .expect("clear");
        assert_eq!(deleted, 2);

        let count = db
            .read(|conn| ClipboardRepository::new(conn).count())
            .expect("count");
        assert_eq!(count, 0);

        let leftover = db
            .read(|conn| ClipboardRepository::new(conn).search("یک", ClipboardFilter::All, None))
            .expect("search after clear");
        assert!(leftover.is_empty());
    }

    #[test]
    fn settings_round_trip() {
        let db = Database::open_in_memory().expect("memory db");
        let updated = db
            .write(|conn| {
                SettingsRepository::new(conn).update(&AppSettings {
                    launch_on_startup: false,
                    enable_monitoring: true,
                    max_history_size: 250,
                })
            })
            .expect("update settings");

        assert!(!updated.launch_on_startup);
        assert!(updated.enable_monitoring);
        assert_eq!(updated.max_history_size, 250);
    }

    #[test]
    fn newest_items_are_returned_first() {
        let db = Database::open_in_memory().expect("memory db");
        save_text(&db, "اول");
        save_text(&db, "دوم");

        let items = db
            .read(|conn| ClipboardRepository::new(conn).list(ClipboardFilter::All, None))
            .expect("list");

        assert_eq!(items[0].content, "دوم");
        assert_eq!(items[1].content, "اول");
    }

    #[test]
    fn evicts_oldest_unpinned_when_over_limit() {
        let db = Database::open_in_memory().expect("memory db");
        save_text(&db, "a");
        save_text(&db, "b");
        save_text(&db, "c");

        let first_id = db
            .read(|conn| ClipboardRepository::new(conn).list(ClipboardFilter::All, None))
            .expect("list")
            .into_iter()
            .last()
            .expect("first inserted")
            .id;

        db.write(|conn| {
            ClipboardRepository::new(conn).set_pinned(&first_id, true)?;
            ClipboardRepository::new(conn).enforce_max_history(2)
        })
        .expect("evict");

        let items = db
            .read(|conn| ClipboardRepository::new(conn).list(ClipboardFilter::All, None))
            .expect("list after evict");

        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|item| item.id == first_id && item.pinned));
    }

    #[test]
    fn prefix_query_finds_persian_word() {
        let db = Database::open_in_memory().expect("memory db");
        save_text(&db, "کتابخانه");
        save_text(&db, "میز");

        let results = db
            .read(|conn| {
                ClipboardRepository::new(conn).search("کتا", ClipboardFilter::All, None)
            })
            .expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "کتابخانه");
    }

    #[test]
    fn multi_word_search_requires_all_terms() {
        let db = Database::open_in_memory().expect("memory db");
        save_text(&db, "hello world from clipoo");
        save_text(&db, "hello there");

        let results = db
            .read(|conn| {
                ClipboardRepository::new(conn).search("hello world", ClipboardFilter::All, None)
            })
            .expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "hello world from clipoo");
    }

    #[test]
    fn search_returns_newest_matches_first() {
        let db = Database::open_in_memory().expect("memory db");
        save_text(&db, "alpha match");
        save_text(&db, "beta match");

        let results = db
            .read(|conn| {
                ClipboardRepository::new(conn).search("match", ClipboardFilter::All, None)
            })
            .expect("search");

        assert_eq!(results[0].content, "beta match");
        assert_eq!(results[1].content, "alpha match");
    }

    #[test]
    fn search_respects_type_filter() {
        let db = Database::open_in_memory().expect("memory db");
        save_text(&db, "https://example.com/docs");
        db.write(|conn| {
            ClipboardRepository::new(conn).insert(NewClipboardItem {
                item_type: ClipboardItemType::Url,
                content: "https://example.com/app".to_string(),
                preview: None,
            })
        })
        .expect("insert url");

        let results = db
            .read(|conn| {
                ClipboardRepository::new(conn).search("example", ClipboardFilter::Url, None)
            })
            .expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item_type, ClipboardItemType::Url);
    }

    #[test]
    fn search_stays_under_50ms_at_50k_rows() {
        use rusqlite::params;
        use std::time::Instant;

        let db = Database::open_in_memory().expect("memory db");
        db.write(|conn| {
            let mut stmt = conn.prepare(
                "INSERT INTO clipboard_items (id, type, content, normalized_content, preview, pinned, created_at)
                 VALUES (?1, 'text', ?2, ?3, ?4, 0, ?5)",
            )?;
            let base_time = 1_700_000_000_000i64;
            for index in 0..50_000i64 {
                let content = format!("row-{index:05} clipboard sample کتاب");
                let normalized = normalize_persian(&content);
                stmt.execute(params![
                    uuid::Uuid::new_v4().to_string(),
                    content,
                    normalized,
                    content,
                    base_time + index
                ])?;
            }
            Ok(())
        })
        .expect("bulk insert");

        let _ = db.read(|conn| {
            ClipboardRepository::new(conn).search("warm", ClipboardFilter::All, Some(10))
        });

        let started = Instant::now();
        let results = db
            .read(|conn| {
                ClipboardRepository::new(conn).search("row-49999", ClipboardFilter::All, Some(50))
            })
            .expect("search");
        let elapsed = started.elapsed();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "row-49999 clipboard sample کتاب");
        assert!(
            elapsed.as_millis() < 50,
            "search took {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn operator_only_search_returns_no_rows() {
        let db = Database::open_in_memory().expect("memory db");
        save_text(&db, "hello");

        let results = db
            .read(|conn| {
                ClipboardRepository::new(conn).search("***", ClipboardFilter::All, None)
            })
            .expect("search");

        assert!(results.is_empty());
    }

    #[test]
    fn eviction_removes_stored_image_files() {
        let dir = std::env::temp_dir().join(format!("clipoo-evict-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let first = dir.join("first.bin");
        let second = dir.join("second.bin");
        std::fs::write(&first, b"one").expect("write first");
        std::fs::write(&second, b"two").expect("write second");

        let db = Database::open_in_memory().expect("memory db");
        db.write(|conn| {
            SettingsRepository::new(conn).update(&AppSettings {
                launch_on_startup: true,
                enable_monitoring: true,
                max_history_size: 1,
            })
        })
        .expect("limit history");

        db.save_clipboard_item(NewClipboardItem {
            item_type: ClipboardItemType::Image,
            content: first.to_string_lossy().into_owned(),
            preview: Some("تصویر".to_string()),
        })
        .expect("first image");
        db.save_clipboard_item(NewClipboardItem {
            item_type: ClipboardItemType::Image,
            content: second.to_string_lossy().into_owned(),
            preview: Some("تصویر".to_string()),
        })
        .expect("second image");

        assert!(!first.exists());
        assert!(second.exists());

        let _ = std::fs::remove_file(&second);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
