use rusqlite::Connection;

const MIGRATIONS: &[(i32, &str)] = &[
    (1, include_str!("../../migrations/001_init.sql")),
    (2, include_str!("../../migrations/002_fts_prefix.sql")),
];

pub fn apply(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        )",
        [],
    )?;

    let applied = load_applied(conn)?;

    for (version, sql) in MIGRATIONS {
        if applied.contains(version) {
            continue;
        }

        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![version, now_millis()],
        )?;
        tx.commit()?;
    }

    Ok(())
}

fn load_applied(conn: &Connection) -> rusqlite::Result<Vec<i32>> {
    let mut stmt = conn.prepare("SELECT version FROM schema_migrations")?;
    let versions = stmt.query_map([], |row| row.get(0))?;
    versions.collect()
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
