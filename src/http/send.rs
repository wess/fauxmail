//! Handlers for sending messages via REST.

use crate::{
  app::AppState,
  http::logs::log_db,
  util::{collect_attachments, collect_headers, extract_bodies},
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use mailparse::parse_mail;
use tracing::error;
use uuid::Uuid;

use super::messages::{SendRequest, SendResponse};

struct NewMessage {
  id: Uuid,
  from: Option<String>,
  to: Vec<String>,
  subject: Option<String>,
  text: Option<String>,
  html: Option<String>,
  headers: std::collections::HashMap<String, String>,
  raw_len: i64,
}

async fn insert_message(state: &AppState, msg: NewMessage) -> Result<(), sqlx::Error> {
  let to_json = serde_json::to_string(&msg.to).unwrap_or_else(|_| "[]".to_string());
  let headers_json = if msg.headers.is_empty() {
    None
  } else {
    Some(serde_json::to_string(&msg.headers).unwrap_or_else(|_| "{}".to_string()))
  };

  sqlx::query(
        "INSERT INTO messages (id, received_at, from_addr, to_recipients, subject, text_body, html_body, headers_json, raw_len) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(msg.id)
    .bind(Utc::now())
    .bind(msg.from)
    .bind(to_json)
    .bind(msg.subject)
    .bind(msg.text)
    .bind(msg.html)
    .bind(headers_json)
    .bind(msg.raw_len)
    .execute(&state.db)
    .await?;
  Ok(())
}

async fn insert_attachments(
  state: &AppState,
  msg_id: Uuid,
  attachments: Vec<(Option<String>, String, Vec<u8>)>,
) -> Result<(), sqlx::Error> {
  for (filename, content_type, data) in attachments {
    let att_id = Uuid::new_v4();
    sqlx::query(
            "INSERT INTO attachments (id, message_id, filename, content_type, size, content) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(att_id)
        .bind(msg_id)
        .bind(filename)
        .bind(content_type)
        .bind(data.len() as i64)
        .bind(data)
        .execute(&state.db)
        .await?;
  }
  Ok(())
}

pub async fn send_message(
  State(state): State<AppState>,
  Json(req): Json<SendRequest>,
) -> impl IntoResponse {
  if req.to.is_empty() {
    return (StatusCode::BAD_REQUEST, "field 'to' must not be empty").into_response();
  }
  let id = Uuid::new_v4();
  let raw_len = 0_i64; // Not applicable for JSON route

  let headers = req.headers.clone();

  if let Err(e) = insert_message(
    &state,
    NewMessage {
      id,
      from: req.from.clone(),
      to: req.to.clone(),
      subject: req.subject.clone(),
      text: req.text.clone(),
      html: req.html.clone(),
      headers,
      raw_len,
    },
  )
  .await
  {
    error!("send_message db error: {e}");
    return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
  }

  log_db(&state, "INFO", &format!("stored message via REST: {id}"))
    .await
    .ok();

  Json(SendResponse { id }).into_response()
}

pub async fn send_raw(State(state): State<AppState>, body: axum::body::Bytes) -> impl IntoResponse {
  let raw = body.to_vec();
  let parsed = match parse_mail(&raw) {
    Ok(p) => p,
    Err(e) => {
      error!("send_raw parse error: {e}");
      return (StatusCode::BAD_REQUEST, "invalid EML").into_response();
    }
  };

  let id = Uuid::new_v4();
  let headers = collect_headers(&parsed);
  let (text, html) = extract_bodies(&parsed);

  let from = headers.get("from").cloned();
  let subject = headers.get("subject").cloned();
  let to: Vec<String> = headers
    .get("to")
    .map(|s| {
      s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
    })
    .unwrap_or_default();

  if let Err(e) = insert_message(
    &state,
    NewMessage {
      id,
      from,
      to,
      subject,
      text,
      html,
      headers,
      raw_len: raw.len() as i64,
    },
  )
  .await
  {
    error!("send_raw db error: {e}");
    return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
  }

  let mut atts = Vec::new();
  collect_attachments(&parsed, &mut atts);
  if let Err(e) = insert_attachments(&state, id, atts).await {
    error!("send_raw attachment insert error: {e}");
    // keep message; attachments are best-effort
  }

  log_db(&state, "INFO", &format!("stored message via EML: {id}"))
    .await
    .ok();

  Json(SendResponse { id }).into_response()
}
