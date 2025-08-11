//! API representation of an email.

use super::db_email::DbEmail;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ApiEmail {
  pub id: Uuid,
  pub received_at: DateTime<Utc>,
  pub from: Option<String>,
  pub to: Vec<String>,
  pub subject: Option<String>,
  pub text: Option<String>,
  pub html: Option<String>,
  pub headers: HashMap<String, String>,
  pub raw_len: i64,
}

impl From<DbEmail> for ApiEmail {
  fn from(d: DbEmail) -> Self {
    let to: Vec<String> = serde_json::from_str(&d.to_recipients).unwrap_or_default();
    let headers: HashMap<String, String> = d
      .headers_json
      .as_deref()
      .and_then(|s| serde_json::from_str(s).ok())
      .unwrap_or_default();
    ApiEmail {
      id: d.id,
      received_at: d.received_at,
      from: d.from_addr,
      to,
      subject: d.subject,
      text: d.text_body,
      html: d.html_body,
      headers,
      raw_len: d.raw_len,
    }
  }
}
