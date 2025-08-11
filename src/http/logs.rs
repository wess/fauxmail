//! Logs API and DB helper.

use crate::{app::AppState, models::log::log_entry::LogEntry};
use axum::{Json, response::IntoResponse};
use chrono::Utc;
use tracing::error;

pub async fn list_logs(
  axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
  let rows: Result<Vec<LogEntry>, _> =
    sqlx::query_as("SELECT id, ts, level, message FROM logs ORDER BY id DESC LIMIT 200")
      .fetch_all(&state.db)
      .await;
  match rows {
    Ok(mut logs) => {
      logs.reverse();
      Json(logs).into_response()
    }
    Err(e) => {
      error!("list_logs error: {e}");
      (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response()
    }
  }
}

pub async fn log_db(state: &AppState, level: &str, message: &str) -> Result<(), sqlx::Error> {
  sqlx::query("INSERT INTO logs (ts, level, message) VALUES (?, ?, ?)")
    .bind(Utc::now())
    .bind(level)
    .bind(message)
    .execute(&state.db)
    .await?;
  Ok(())
}
