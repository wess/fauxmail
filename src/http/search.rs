//! Search API.

use crate::{
  app::AppState,
  http::messages::ListParams,
  models::email::{api_email::ApiEmail, db_email::DbEmail},
};
use axum::{Json, extract::Query, response::IntoResponse};
use std::collections::HashMap;
use tracing::error;

pub async fn search_messages(
  axum::extract::State(state): axum::extract::State<AppState>,
  Query(params): Query<HashMap<String, String>>,
) -> axum::response::Response {
  let q = params.get("q").map(|s| s.trim()).filter(|s| !s.is_empty());
  if q.is_none() {
    return super::messages::list_messages(
      axum::extract::State(state),
      Query(ListParams::default()),
    )
    .await
    .into_response();
  }
  let like = format!("%{}%", q.unwrap());
  let rows: Result<Vec<DbEmail>, _> = sqlx::query_as("SELECT id, received_at, from_addr, to_recipients, subject, text_body, html_body, headers_json, raw_len FROM messages WHERE coalesce(from_addr,'') LIKE ? OR coalesce(subject,'') LIKE ? OR coalesce(text_body,'') LIKE ? OR to_recipients LIKE ? ORDER BY received_at DESC LIMIT 200")
        .bind(&like).bind(&like).bind(&like).bind(&like).fetch_all(&state.db).await;
  match rows {
    Ok(rows) => {
      let out: Vec<ApiEmail> = rows.into_iter().map(ApiEmail::from).collect();
      Json(out).into_response()
    }
    Err(e) => {
      error!("search error: {e}");
      (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response()
    }
  }
}
