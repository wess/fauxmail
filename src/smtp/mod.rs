//! Minimal SMTP listener for local development.
//!
//! Supports HELO/EHLO, optional AUTH LOGIN/PLAIN, MAIL FROM, RCPT TO, DATA, QUIT.

use crate::{
    app::AppState,
    http::logs::log_db,
    util::{collect_attachments, collect_headers, extract_bodies},
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use chrono::Utc;
use mailparse::parse_mail;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub async fn start_smtp(state: AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = std::env::var("FAUXMAIL_SMTP_ADDR").unwrap_or_else(|_| "127.0.0.1:1025".to_string());
    let listener = TcpListener::bind(&addr).await?;
    info!("smtp listener: {}", addr);

    loop {
        let (stream, peer) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(state, stream).await {
                warn!("smtp connection error from {}: {}", peer, e);
            }
        });
    }
}

async fn handle_client(
    state: AppState,
    stream: TcpStream,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user = std::env::var("FAUXMAIL_SMTP_USER").ok();
    let pass = std::env::var("FAUXMAIL_SMTP_PASS").ok();
    let require_auth = user.is_some() && pass.is_some();

    let (read_half, mut writer) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    writer.write_all(b"220 fauxmail dev smtp\r\n").await?;
    writer.flush().await?;

    let mut authed = !require_auth;
    let mut mail_from: Option<String> = None;
    let mut rcpts: Vec<String> = Vec::new();
    let mut buf = String::new();

    loop {
        buf.clear();
        let n = reader.read_line(&mut buf).await?;
        if n == 0 {
            break;
        }
        let line = buf.trim_end_matches(['\r', '\n']);
        debug!("smtp <= {}", line);
        let upper = line.to_uppercase();

        if upper.starts_with("EHLO") || upper.starts_with("HELO") {
            writer.write_all(b"250-fauxmail\r\n").await?;
            if require_auth {
                writer.write_all(b"250-AUTH PLAIN LOGIN\r\n").await?;
            }
            writer.write_all(b"250 OK\r\n").await?;
        } else if upper.starts_with("AUTH ") {
            if !require_auth {
                writer.write_all(b"503 AUTH not required\r\n").await?;
                continue;
            }
            if upper.starts_with("AUTH LOGIN") {
                writer.write_all(b"334 VXNlcm5hbWU6\r\n").await?; // 'Username:'
                let mut u = String::new();
                reader.read_line(&mut u).await?;
                let u = u.trim_end_matches(['\r', '\n']);
                let Ok(decoded_user) = String::from_utf8(B64.decode(u)?) else {
                    writer.write_all(b"535 auth failed\r\n").await?;
                    continue;
                };
                writer.write_all(b"334 UGFzc3dvcmQ6\r\n").await?; // 'Password:'
                let mut p = String::new();
                reader.read_line(&mut p).await?;
                let p = p.trim_end_matches(['\r', '\n']);
                let Ok(decoded_pass) = String::from_utf8(B64.decode(p)?) else {
                    writer.write_all(b"535 auth failed\r\n").await?;
                    continue;
                };
                if decoded_user == user.clone().unwrap() && decoded_pass == pass.clone().unwrap() {
                    authed = true;
                    writer
                        .write_all(b"235 Authentication successful\r\n")
                        .await?;
                } else {
                    writer.write_all(b"535 Authentication failed\r\n").await?;
                }
            } else if upper.starts_with("AUTH PLAIN") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let token = if parts.len() >= 3 { parts[2] } else { "" };
                let data = B64.decode(token)?;
                // format: "\0username\0password"
                let mut iter = data.split(|b| *b == 0);
                let _ = iter.next();
                let u =
                    String::from_utf8(iter.next().unwrap_or_default().to_vec()).unwrap_or_default();
                let p =
                    String::from_utf8(iter.next().unwrap_or_default().to_vec()).unwrap_or_default();
                if Some(u) == user.clone() && Some(p) == pass.clone() {
                    authed = true;
                    writer
                        .write_all(b"235 Authentication successful\r\n")
                        .await?;
                } else {
                    writer.write_all(b"535 Authentication failed\r\n").await?;
                }
            } else {
                writer
                    .write_all(b"504 Unrecognized authentication type\r\n")
                    .await?;
            }
        } else if upper.starts_with("MAIL FROM:") {
            if require_auth && !authed {
                writer.write_all(b"530 Authentication required\r\n").await?;
                continue;
            }
            mail_from = Some(line[10..].trim().trim_matches(['<', '>']).to_string());
            rcpts.clear();
            writer.write_all(b"250 OK\r\n").await?;
        } else if upper.starts_with("RCPT TO:") {
            if require_auth && !authed {
                writer.write_all(b"530 Authentication required\r\n").await?;
                continue;
            }
            rcpts.push(line[8..].trim().trim_matches(['<', '>']).to_string());
            writer.write_all(b"250 Accepted\r\n").await?;
        } else if upper == "DATA" {
            if require_auth && !authed {
                writer.write_all(b"530 Authentication required\r\n").await?;
                continue;
            }
            writer
                .write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n")
                .await?;
            let mut data = Vec::new();
            // Read until line with single '.'
            loop {
                let mut line = String::new();
                let n = reader.read_line(&mut line).await?;
                if n == 0 {
                    break;
                }
                if line == ".\r\n" || line == ".\n" {
                    break;
                }
                data.extend_from_slice(line.as_bytes());
            }

            // Store message
            let id = Uuid::new_v4();
            match store_raw_message(&state, id, mail_from.clone(), rcpts.clone(), data).await {
                Ok(_) => {
                    let _ =
                        log_db(&state, "INFO", &format!("stored message via SMTP: {}", id)).await;
                    writer
                        .write_all(format!("250 OK id={}\r\n", id).as_bytes())
                        .await?;
                }
                Err(e) => {
                    error!("smtp store error: {e}");
                    writer
                        .write_all(b"451 Requested action aborted: local error\r\n")
                        .await?;
                }
            }
        } else if upper == "RSET" {
            mail_from = None;
            rcpts.clear();
            writer.write_all(b"250 OK\r\n").await?;
        } else if upper == "NOOP" {
            writer.write_all(b"250 OK\r\n").await?;
        } else if upper == "QUIT" {
            writer.write_all(b"221 Bye\r\n").await?;
            break;
        } else {
            writer.write_all(b"502 Command not implemented\r\n").await?;
        }
    }
    Ok(())
}

async fn store_raw_message(
    state: &AppState,
    id: Uuid,
    from: Option<String>,
    to: Vec<String>,
    raw: Vec<u8>,
) -> Result<(), sqlx::Error> {
    let parsed = parse_mail(&raw).map_err(|e| {
        error!("smtp parse error: {e}");
        sqlx::Error::Protocol("parse error".into())
    })?;
    let (text, html) = extract_bodies(&parsed);
    let mut headers = collect_headers(&parsed);
    if !headers.contains_key("from") {
        if let Some(f) = from.clone() {
            headers.insert("from".into(), f);
        }
    }
    if !headers.contains_key("to") && !to.is_empty() {
        headers.insert("to".into(), to.join(", "));
    }
    let subject = headers.get("subject").cloned();

    // Insert message
    let to_json = serde_json::to_string(&to).unwrap_or_else(|_| "[]".to_string());
    let headers_json = if headers.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&headers).unwrap_or_else(|_| "{}".to_string()))
    };
    sqlx::query(
        "INSERT INTO messages (id, received_at, from_addr, to_recipients, subject, text_body, html_body, headers_json, raw_len) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(Utc::now())
    .bind(from)
    .bind(to_json)
    .bind(subject)
    .bind(text)
    .bind(html)
    .bind(headers_json)
    .bind(raw.len() as i64)
    .execute(&state.db)
    .await?;

    // Attachments
    let mut atts = Vec::new();
    collect_attachments(&parsed, &mut atts);
    for (filename, content_type, data) in atts {
        let att_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO attachments (id, message_id, filename, content_type, size, content) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(att_id)
        .bind(id)
        .bind(filename)
        .bind(content_type)
        .bind(data.len() as i64)
        .bind(data)
        .execute(&state.db)
        .await?;
    }
    Ok(())
}
