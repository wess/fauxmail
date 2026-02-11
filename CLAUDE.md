# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is fauxmail

A lightweight local mock email server written in Rust. It provides an HTTP API (Axum), an SMTP server, and a web dashboard, all backed by SQLite.

## Build & Development Commands

```bash
cargo build                              # Debug build
cargo build --release                    # Release build
cargo test --all                         # Run all integration tests
cargo fmt --all -- --check               # Check formatting
cargo clippy --all-targets -- -D warnings # Lint (warnings are errors)
cargo run                                # Run locally
```

Tests are integration tests in `tests/app.rs` using in-memory SQLite and reqwest.

## Environment Variables

- `FAUXMAIL_ADDR` — HTTP bind address (default `127.0.0.1:8025`)
- `FAUXMAIL_SMTP_ADDR` — SMTP bind address (default `127.0.0.1:1025`)
- `FAUXMAIL_DATABASE` — SQLite URL (default `sqlite://fauxmail.db`)
- `FAUXMAIL_SMTP_USER` / `FAUXMAIL_SMTP_PASS` — Optional SMTP auth credentials
- `RUST_LOG` — Tracing filter (default `info`)

## Architecture

Three interfaces feed into a shared SQLite database:

- **HTTP API** (`src/http/`) — Axum router with REST endpoints for sending (JSON and raw EML), listing, searching, clearing messages, downloading attachments, and streaming logs
- **SMTP Server** (`src/smtp/`) — Async TCP listener handling EHLO, AUTH LOGIN/PLAIN, MAIL FROM/RCPT TO/DATA, with mailparse for MIME extraction
- **Web Dashboard** (`GET /`) — Server-rendered HTML with embedded JS for live log polling and message browsing

Shared state is `AppState { db: SqlitePool }` passed through Axum's state extractor.

## Module Layout

- `src/app/` — Entry point, server startup, AppState
- `src/http/` — Route builder and handlers (send, messages, search, logs, attachments, ui)
- `src/smtp/` — SMTP listener and per-connection handler
- `src/db/` — Migrations and SQLite path setup
- `src/models/` — Separated into api vs db structs: `email/`, `attachment/`, `log/`, `response/`
- `src/util/` — Tracing init, HTML escaping, MIME helpers (header/body/attachment extraction)

## Key Patterns

- No ORM — raw sqlx queries with compile-time checked macros
- Models split into `db_*` (FromRow for SELECT) and `api_*` (Serialize for JSON responses)
- MIME parsing shared between SMTP and HTTP raw endpoint via `src/util/` helpers
- All message IDs are UUIDs generated server-side
- Database has three tables: `messages`, `attachments`, `logs`

## Formatting

Uses 2-space indentation (`tab_spaces=2` in `rustfmt.toml`), field init shorthand, and try shorthand.
