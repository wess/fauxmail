//! Database helpers: migrations and path handling.

use sqlx::SqlitePool;
use std::path::Path;

/// Run SQLite migrations to create tables if absent.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            received_at TEXT NOT NULL,
            from_addr TEXT NULL,
            to_recipients TEXT NOT NULL,
            subject TEXT NULL,
            text_body TEXT NULL,
            html_body TEXT NULL,
            headers_json TEXT NULL,
            raw_len INTEGER NOT NULL
        )"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ts TEXT NOT NULL,
            level TEXT NOT NULL,
            message TEXT NOT NULL
        )"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS attachments (
            id TEXT PRIMARY KEY,
            message_id TEXT NOT NULL,
            filename TEXT NULL,
            content_type TEXT NOT NULL,
            size INTEGER NOT NULL,
            content BLOB NOT NULL
        )"#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Ensure SQLite file and parent folder exist for a given sqlx URL.
pub fn ensure_sqlite_path(db_url: &str) -> String {
    if !db_url.starts_with("sqlite:") {
        return db_url.to_string();
    }
    let path_part = db_url.trim_start_matches("sqlite://");
    if path_part == ":memory:" {
        return db_url.to_string();
    }
    let (path_only, _) = match path_part.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (path_part, None),
    };
    if !path_only.is_empty() {
        let p = Path::new(path_only);
        if let Some(parent) = p.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p);
    }
    db_url.to_string()
}
