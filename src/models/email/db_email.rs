//! Database row for an email.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct DbEmail {
    pub id: Uuid,
    pub received_at: DateTime<Utc>,
    pub from_addr: Option<String>,
    pub to_recipients: String,
    pub subject: Option<String>,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub headers_json: Option<String>,
    pub raw_len: i64,
}
