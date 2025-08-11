//! fauxmail library entrypoint.
//!
//! Modules:
//! - `app`: startup, configuration, shared state
//! - `http`: Axum router and handlers
//! - `smtp`: lightweight SMTP listener (local dev)
//! - `db`: migrations and SQLite helpers
//! - `models`: typed records used across layers
//! - `util`: helpers for parsing and HTML escaping

pub mod app;
pub mod db;
pub mod http;
pub mod models;
pub mod smtp;
pub mod util;
