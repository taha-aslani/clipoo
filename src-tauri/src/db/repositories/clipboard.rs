use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::error::{DbError, DbResult};
use crate::db::fts::to_match_query;
use crate::db::models::{
    ClipboardFilter, ClipboardItem, ClipboardItemType, NewClipboardItem, SaveOutcome,
};
use crate::persian::normalize_persian;

const DEFAULT_LIMIT: u32 = 100;
const PREVIEW_CHARS: usize = 240;

pub struct ClipboardRepository<'a> {
    conn: &'a Connection,
}

impl<'a> ClipboardRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn insert(&self, new_item: NewClipboardItem) -> DbResult<SaveOutcome> {
        let normalized_content = normalize_persian(&new_item.content);

        if self.is_consecutive_duplicate(&normalized_content)? {
            return Ok(SaveOutcome::Duplicate);
        }

        let item = ClipboardItem {
            id: uuid::Uuid::new_v4().to_string(),
            item_type: new_item.item_type,
            preview: new_item
                .preview
                .unwrap_or_else(|| make_preview(&new_item.content)),
            content: new_item.content,
            normalized_content,
            pinned: false,
            created_at: now_millis(),
        };

        self.conn.execute(
            "INSERT INTO clipboard_items (id, type, content, normalized_content, preview, pinned, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                item.id,
                item.item_type.as_str(),
                item.content,
                item.normalized_content,
                item.preview,
                i32::from(item.pinned),
                item.created_at
            ],
        )?;

        Ok(SaveOutcome::Inserted(item))
    }

    pub fn is_consecutive_duplicate(&self, normalized_content: &str) -> DbResult<bool> {
        let latest: Option<String> = self
            .conn
            .query_row(
                "SELECT normalized_content FROM clipboard_items ORDER BY created_at DESC, rowid DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;

        Ok(latest.is_some_and(|value| value == normalized_content))
    }

    pub fn list(&self, filter: ClipboardFilter, limit: Option<u32>) -> DbResult<Vec<ClipboardItem>> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).max(1);
        let mut stmt = self.conn.prepare(
            "SELECT id, type, content, normalized_content, preview, pinned, created_at
             FROM clipboard_items
             WHERE (?1 = 'all')
                OR (?1 = 'pinned' AND pinned = 1)
                OR (?1 = type)
             ORDER BY created_at DESC, rowid DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![filter_sql(filter), limit], map_item)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    pub fn search(
        &self,
        query: &str,
        filter: ClipboardFilter,
        limit: Option<u32>,
    ) -> DbResult<Vec<ClipboardItem>> {
        let Some(match_query) = to_match_query(query) else {
            if query.trim().is_empty() {
                return self.list(filter, limit);
            }
            return Ok(Vec::new());
        };

        let limit = limit.unwrap_or(DEFAULT_LIMIT).max(1);
        let mut stmt = self.conn.prepare(
            "SELECT i.id, i.type, i.content, i.normalized_content, i.preview, i.pinned, i.created_at
             FROM clipboard_items_fts
             JOIN clipboard_items AS i ON i.rowid = clipboard_items_fts.rowid
             WHERE clipboard_items_fts MATCH ?1
               AND (
                    ?2 = 'all'
                    OR (?2 = 'pinned' AND i.pinned = 1)
                    OR ?2 = i.type
               )
             ORDER BY i.created_at DESC, i.rowid DESC
             LIMIT ?3",
        )?;

        let rows = stmt.query_map(params![match_query, filter_sql(filter), limit], map_item);
        match rows {
            Ok(mapped) => match mapped.collect::<Result<Vec<_>, _>>() {
                Ok(items) => Ok(items),
                Err(error) if is_fts_query_error(&error) => Ok(Vec::new()),
                Err(error) => Err(DbError::from(error)),
            },
            Err(error) if is_fts_query_error(&error) => Ok(Vec::new()),
            Err(error) => Err(DbError::from(error)),
        }
    }

    pub fn set_pinned(&self, id: &str, pinned: bool) -> DbResult<ClipboardItem> {
        let changed = self.conn.execute(
            "UPDATE clipboard_items SET pinned = ?1 WHERE id = ?2",
            params![i32::from(pinned), id],
        )?;

        if changed == 0 {
            return Err(DbError::NotFound);
        }

        self.get(id)
    }

    pub fn get(&self, id: &str) -> DbResult<ClipboardItem> {
        self.conn
            .query_row(
                "SELECT id, type, content, normalized_content, preview, pinned, created_at
                 FROM clipboard_items WHERE id = ?1",
                params![id],
                map_item,
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
                other => DbError::from(other),
            })
    }

    pub fn delete(&self, id: &str) -> DbResult<()> {
        let changed = self
            .conn
            .execute("DELETE FROM clipboard_items WHERE id = ?1", params![id])?;

        if changed == 0 {
            return Err(DbError::NotFound);
        }

        Ok(())
    }

    pub fn clear(&self) -> DbResult<usize> {
        let deleted = self.conn.execute("DELETE FROM clipboard_items", [])?;
        Ok(deleted)
    }

    pub fn count(&self) -> DbResult<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM clipboard_items", [], |row| row.get(0))
            .map_err(DbError::from)
    }

    pub fn enforce_max_history(&self, max_history_size: u32) -> DbResult<Vec<ClipboardItem>> {
        let max_history_size = max_history_size.max(1) as i64;
        let count = self.count()?;
        let overflow = count.saturating_sub(max_history_size);

        if overflow <= 0 {
            return Ok(Vec::new());
        }

        let mut stmt = self.conn.prepare(
            "SELECT id, type, content, normalized_content, preview, pinned, created_at
             FROM clipboard_items
             WHERE pinned = 0
             ORDER BY created_at ASC, rowid ASC
             LIMIT ?1",
        )?;
        let victims = stmt
            .query_map(params![overflow], map_item)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::from)?;

        if victims.is_empty() {
            return Ok(Vec::new());
        }

        self.conn.execute(
            "DELETE FROM clipboard_items
             WHERE id IN (
                SELECT id FROM clipboard_items
                WHERE pinned = 0
                ORDER BY created_at ASC, rowid ASC
                LIMIT ?1
             )",
            params![overflow],
        )?;

        Ok(victims)
    }
}

fn is_fts_query_error(error: &rusqlite::Error) -> bool {
    let message = error.to_string();
    message.contains("fts5") || message.contains("no such column")
}

fn filter_sql(filter: ClipboardFilter) -> &'static str {
    match filter {
        ClipboardFilter::All => "all",
        ClipboardFilter::Text => "text",
        ClipboardFilter::Image => "image",
        ClipboardFilter::File => "file",
        ClipboardFilter::Url => "url",
        ClipboardFilter::Pinned => "pinned",
    }
}

fn map_item(row: &Row<'_>) -> rusqlite::Result<ClipboardItem> {
    let type_value = row.get::<_, String>(1)?;
    let item_type = ClipboardItemType::parse(&type_value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            1,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid clipboard type",
            )),
        )
    })?;

    Ok(ClipboardItem {
        id: row.get(0)?,
        item_type,
        content: row.get(2)?,
        normalized_content: row.get(3)?,
        preview: row.get(4)?,
        pinned: row.get::<_, i32>(5)? != 0,
        created_at: row.get(6)?,
    })
}

fn make_preview(content: &str) -> String {
    content.chars().take(PREVIEW_CHARS).collect()
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
