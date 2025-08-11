//! Log entry stored in SQLite and exposed via API.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct LogEntry {
    pub id: i64,
    pub ts: DateTime<Utc>,
    pub level: String,
    pub message: String,
}
