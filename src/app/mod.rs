//! Application setup and runtime.

use crate::{db, http, smtp};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::net::SocketAddr;
use tracing::{error, info};

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
  pub db: SqlitePool,
}

/// Start HTTP and SMTP servers with configured environment.
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  crate::util::init_tracing();

  let db_url =
    std::env::var("FAUXMAIL_DATABASE").unwrap_or_else(|_| "sqlite://fauxmail.db".to_string());
  let db_url = db::ensure_sqlite_path(&db_url);
  let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect(&db_url)
    .await?;
  db::run_migrations(&pool).await?;

  let state = AppState { db: pool.clone() };

  let app = http::build_router(state.clone());

  let addr: SocketAddr = std::env::var("FAUXMAIL_ADDR")
    .unwrap_or_else(|_| "127.0.0.1:8025".to_string())
    .parse()?;

  info!("fauxmail dashboard:    http://{}/", addr);
  info!("REST send endpoint:   POST http://{}/send", addr);
  info!("Raw EML endpoint:     POST http://{}/send/raw", addr);

  // Start SMTP listener in background
  let smtp_state = state.clone();
  tokio::spawn(async move {
    if let Err(e) = smtp::start_smtp(smtp_state).await {
      error!("smtp listener error: {e}");
    }
  });

  // Start HTTP server
  let listener = tokio::net::TcpListener::bind(addr).await?;
  axum::serve(listener, app).await?;
  Ok(())
}
