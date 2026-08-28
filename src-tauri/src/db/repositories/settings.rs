use rusqlite::{params, Connection};

use crate::db::error::{DbError, DbResult};
use crate::db::models::AppSettings;

pub struct SettingsRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SettingsRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get(&self) -> DbResult<AppSettings> {
        self.conn
            .query_row(
                "SELECT launch_on_startup, enable_monitoring, max_history_size
                 FROM settings WHERE id = 1",
                [],
                |row| {
                    Ok(AppSettings {
                        launch_on_startup: row.get::<_, i32>(0)? != 0,
                        enable_monitoring: row.get::<_, i32>(1)? != 0,
                        max_history_size: row.get::<_, u32>(2)?,
                    })
                },
            )
            .map_err(DbError::from)
    }

    pub fn update(&self, settings: &AppSettings) -> DbResult<AppSettings> {
        let max_history_size = settings.max_history_size.max(1);

        self.conn.execute(
            "UPDATE settings
             SET launch_on_startup = ?1,
                 enable_monitoring = ?2,
                 max_history_size = ?3
             WHERE id = 1",
            params![
                i32::from(settings.launch_on_startup),
                i32::from(settings.enable_monitoring),
                max_history_size
            ],
        )?;

        self.get()
    }
}
