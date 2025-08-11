//! Attachments API.

use crate::{
  app::AppState,
  models::attachment::{attachment_meta::AttachmentMeta, attachment_row::AttachmentRow},
};
use axum::{
  Json,
  extract::Path as AxumPath,
  http::{HeaderMap, StatusCode, header},
  response::IntoResponse,
};
use tracing::error;
use uuid::Uuid;

pub async fn list_attachments(
  axum::extract::State(state): axum::extract::State<AppState>,
  AxumPath(id): AxumPath<Uuid>,
) -> impl IntoResponse {
  let rows: Result<Vec<AttachmentMeta>, _> = sqlx::query_as("SELECT id, message_id, filename, content_type, size FROM attachments WHERE message_id = ? ORDER BY id").bind(id).fetch_all(&state.db).await;
  match rows {
    Ok(v) => Json(v).into_response(),
    Err(e) => {
      error!("list_attachments error: {e}");
      (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response()
    }
  }
}

pub async fn download_attachment(
  axum::extract::State(state): axum::extract::State<AppState>,
  AxumPath(att_id): AxumPath<Uuid>,
) -> impl IntoResponse {
  let row: Result<Option<AttachmentRow>, _> =
    sqlx::query_as("SELECT id, filename, content_type, content FROM attachments WHERE id = ?")
      .bind(att_id)
      .fetch_optional(&state.db)
      .await;
  match row {
    Ok(Some(a)) => {
      let mut headers = HeaderMap::new();
      headers.insert(
        header::CONTENT_TYPE,
        a.content_type
          .parse()
          .unwrap_or("application/octet-stream".parse().unwrap()),
      );
      if let Some(name) = a.filename {
        headers.insert(
          header::CONTENT_DISPOSITION,
          format!("inline; filename=\"{}\"", name).parse().unwrap(),
        );
      }
      (headers, a.content).into_response()
    }
    Ok(None) => (StatusCode::NOT_FOUND, "not found").into_response(),
    Err(e) => {
      error!("download_attachment error: {e}");
      (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response()
    }
  }
}
