//! Message JSON APIs and HTML message view.

use crate::{
  app::AppState,
  models::{
    attachment::attachment_meta::AttachmentMeta,
    email::{api_email::ApiEmail, db_email::DbEmail},
    response::message_with_attachments::MessageWithAttachments,
  },
  util::html_escape,
};
use axum::{
  Json,
  extract::{Path as AxumPath, Query, State},
  http::{HeaderMap, StatusCode, header},
  response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SendRequest {
  pub from: Option<String>,
  pub to: Vec<String>,
  pub subject: Option<String>,
  pub text: Option<String>,
  pub html: Option<String>,
  #[serde(default)]
  pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct SendResponse {
  pub id: Uuid,
}

#[derive(Debug, Default, Deserialize)]
pub struct ListParams {
  pub page: Option<u32>,
  pub limit: Option<u32>,
  pub sort: Option<String>,
  pub dir: Option<String>,
  pub q: Option<String>,
}

pub fn compute_list_params(
  p: &ListParams,
) -> (u32, u32, &'static str, &'static str, Option<String>) {
  let page = p.page.unwrap_or(1).max(1);
  let limit = p.limit.unwrap_or(50).clamp(1, 200);
  let offset = (page - 1) * limit;
  let order_by = match p.sort.as_deref() {
    Some("subject") => "subject",
    Some("from") => "from_addr",
    Some("to") => "to_recipients",
    _ => "received_at",
  };
  let dir = match p.dir.as_deref() {
    Some("asc") => "ASC",
    _ => "DESC",
  };
  let like = p.q.as_ref().and_then(|s| {
    let t = s.trim();
    if t.is_empty() {
      None
    } else {
      Some(format!("%{}%", t))
    }
  });
  (limit, offset, order_by, dir, like)
}

pub async fn list_messages(
  State(state): State<AppState>,
  Query(params): Query<ListParams>,
) -> impl IntoResponse {
  let (limit, offset, order_by, dir, like) = compute_list_params(&params);
  let sql = if like.is_some() {
    format!(
      "SELECT id, received_at, from_addr, to_recipients, subject, text_body, html_body, headers_json, raw_len FROM messages WHERE coalesce(from_addr,'') LIKE ? OR coalesce(subject,'') LIKE ? OR coalesce(text_body,'') LIKE ? OR to_recipients LIKE ? ORDER BY {} {} LIMIT ? OFFSET ?",
      order_by, dir
    )
  } else {
    format!(
      "SELECT id, received_at, from_addr, to_recipients, subject, text_body, html_body, headers_json, raw_len FROM messages ORDER BY {} {} LIMIT ? OFFSET ?",
      order_by, dir
    )
  };
  let mut query = sqlx::query_as::<_, DbEmail>(&sql);
  if let Some(like_val) = like.as_ref() {
    query = query
      .bind(like_val)
      .bind(like_val)
      .bind(like_val)
      .bind(like_val);
  }
  match query
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await
  {
    Ok(rows) => {
      let out: Vec<ApiEmail> = rows.into_iter().map(ApiEmail::from).collect();
      Json(out).into_response()
    }
    Err(e) => {
      error!("list_messages error: {e}");
      (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response()
    }
  }
}

pub async fn clear_messages(State(state): State<AppState>) -> impl IntoResponse {
  if let Err(e) = sqlx::query("DELETE FROM messages").execute(&state.db).await {
    error!("clear_messages error: {e}");
    return StatusCode::INTERNAL_SERVER_ERROR;
  }
  crate::http::logs::log_db(&state, "INFO", "cleared all messages")
    .await
    .ok();
  StatusCode::NO_CONTENT
}

pub async fn get_message(
  State(state): State<AppState>,
  AxumPath(id): AxumPath<Uuid>,
) -> impl IntoResponse {
  let row = sqlx::query_as::<_, DbEmail>("SELECT id, received_at, from_addr, to_recipients, subject, text_body, html_body, headers_json, raw_len FROM messages WHERE id = ?").bind(id).fetch_optional(&state.db).await;
  match row {
    Ok(Some(m)) => {
      let attachments: Vec<AttachmentMeta> = sqlx::query_as("SELECT id, message_id, filename, content_type, size FROM attachments WHERE message_id = ? ORDER BY id").bind(id).fetch_all(&state.db).await.unwrap_or_default();
      Json(MessageWithAttachments {
        message: ApiEmail::from(m),
        attachments,
      })
      .into_response()
    }
    Ok(None) => (StatusCode::NOT_FOUND, "message not found").into_response(),
    Err(e) => {
      error!("get_message error: {e}");
      (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response()
    }
  }
}

pub async fn get_message_html(
  State(state): State<AppState>,
  AxumPath(id): AxumPath<Uuid>,
) -> impl IntoResponse {
  let row = sqlx::query_as::<_, DbEmail>("SELECT id, received_at, from_addr, to_recipients, subject, text_body, html_body, headers_json, raw_len FROM messages WHERE id = ?").bind(id).fetch_optional(&state.db).await.ok().flatten();
  if let Some(m) = row {
    let html = m
      .html_body
      .clone()
      .or_else(|| {
        m.text_body
          .clone()
          .map(|t| format!("<pre>{}</pre>", html_escape(&t)))
      })
      .unwrap_or_else(|| "<em>No content</em>".to_string());
    let tmpl = r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>{SUBJECT}</title>
<style>body { font-family: system-ui, sans-serif; margin: 1.5rem; }</style>
</head>
<body>
  <p><a href="/">← back</a></p>
  <h2>{SUBJECT}</h2>
  <p><strong>From:</strong> {FROM} &nbsp; <strong>To:</strong> {TO}</p>
  <hr/>
  <div>{HTML}</div>
  <h3>Attachments</h3>
  <div id="atts"></div>
  <script>
    (async () => {
      const res = await fetch('/messages/{ID}/attachments');
      const atts = await res.json();
      const el = document.getElementById('atts');
      if (!atts.length) { el.textContent = 'None'; return; }
      el.innerHTML = atts.map(a => {
        if (a.content_type.startsWith('image/')) {
          return `<div><a href="/attachments/${'${'}a.id}/download" target="_blank">${'${'}a.filename || '(image)'} (${'${'}a.size} bytes)</a><br/><img src="/attachments/${'${'}a.id}/download" style="max-width:600px;max-height:400px;border:1px solid #ddd;margin:.5rem 0;"/></div>`;
        }
        return `<div><a href="/attachments/${'${'}a.id}/download" target="_blank">${'${'}a.filename || '(attachment)'} (${'${'}a.size} bytes, ${'${'}a.content_type})</a></div>`;
      }).join('');
    })();
  </script>
</body></html>"#;
    let page = tmpl
      .replace(
        "{SUBJECT}",
        &m.subject.clone().unwrap_or_else(|| "(no subject)".into()),
      )
      .replace(
        "{FROM}",
        &m.from_addr.clone().unwrap_or_else(|| "(unknown)".into()),
      )
      .replace("{TO}", &{
        let v: Vec<String> = serde_json::from_str(&m.to_recipients).unwrap_or_default();
        if v.is_empty() {
          "(none)".into()
        } else {
          v.join(", ")
        }
      })
      .replace("{HTML}", &html)
      .replace("{ID}", &m.id.to_string());
    let mut headers = HeaderMap::new();
    headers.insert(
      header::CONTENT_TYPE,
      "text/html; charset=utf-8".parse().unwrap(),
    );
    return (headers, page).into_response();
  }
  (StatusCode::NOT_FOUND, "message not found").into_response()
}
