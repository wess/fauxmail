//! HTTP router and handlers.

use crate::app::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub mod attachments;
pub mod logs;
pub mod messages;
pub mod search;
pub mod send;
pub mod ui;

/// Assemble the HTTP router with all routes.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(ui::ui_index))
        .route(
            "/messages",
            get(messages::list_messages).delete(messages::clear_messages),
        )
        .route("/messages/:id", get(messages::get_message))
        .route("/messages/:id/html", get(messages::get_message_html))
        .route(
            "/messages/:id/attachments",
            get(attachments::list_attachments),
        )
        .route(
            "/attachments/:att_id/download",
            get(attachments::download_attachment),
        )
        .route("/search", get(search::search_messages))
        .route("/send", post(send::send_message))
        .route("/send/raw", post(send::send_raw))
        .route("/logs", get(logs::list_logs))
        .with_state(state)
}
