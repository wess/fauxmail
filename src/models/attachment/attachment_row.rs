//! Attachment row for downloads.

use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct AttachmentRow {
  pub id: Uuid,
  pub filename: Option<String>,
  pub content_type: String,
  pub content: Vec<u8>,
}
