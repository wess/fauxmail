//! Public attachment metadata.

use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct AttachmentMeta {
  pub id: Uuid,
  pub message_id: Uuid,
  pub filename: Option<String>,
  pub content_type: String,
  pub size: i64,
}
