//! Dashboard HTML.

use crate::{app::AppState, http::messages::ListParams, models::email::db_email::DbEmail};
use axum::{extract::Query, response::Html};

pub async fn ui_index(
  axum::extract::State(state): axum::extract::State<AppState>,
  Query(params): Query<ListParams>,
) -> Html<String> {
  let (limit, offset, order_by, dir, like) = super::messages::compute_list_params(&params);
  let sql = if like.is_some() {
    format!(
      "SELECT id, received_at, from_addr, to_recipients, subject, text_body, html_body, headers_json, raw_len FROM messages WHERE coalesce(from_addr,'') LIKE ? OR coalesce(subject,'') LIKE ? OR coalesce(text_body,'') LIKE ? OR to_recipients LIKE ? ORDER BY {order_by} {dir} LIMIT ? OFFSET ?"
    )
  } else {
    format!(
      "SELECT id, received_at, from_addr, to_recipients, subject, text_body, html_body, headers_json, raw_len FROM messages ORDER BY {order_by} {dir} LIMIT ? OFFSET ?"
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
  let msgs: Vec<DbEmail> = query
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

  let mut rows = String::new();
  for d in msgs.iter() {
    let subj = d.subject.as_deref().unwrap_or("(no subject)");
    let from = d.from_addr.as_deref().unwrap_or("(unknown)");
    let to_list: Vec<String> = serde_json::from_str(&d.to_recipients).unwrap_or_default();
    let to = if to_list.is_empty() {
      "(none)".to_string()
    } else {
      to_list.join(", ")
    };
    rows.push_str(&format!(
            "<tr><td><a href=\"/messages/{id}/html\">{id}</a></td><td>{when}</td><td>{from}</td><td>{to}</td><td>{subj}</td></tr>",
            id = d.id,
            when = d.received_at
        ));
  }
  let template = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>fauxmail</title>
  <style>
    body { font-family: system-ui, sans-serif; margin: 2rem; }
    h1 { margin: 0 0 1rem 0; }
    table { width: 100%; border-collapse: collapse; }
    th, td { border-bottom: 1px solid #ddd; text-align: left; padding: .5rem; }
    .actions { margin: 1rem 0; }
    code { background: #f6f8fa; padding: .2rem .4rem; border-radius: 4px; }
    .logs { background:#0b1020; color:#e6edf3; padding:1rem; border-radius:8px; white-space:pre-wrap; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; font-size: 12px; }
    .lvl-INFO { color:#7ee787; }
    .lvl-ERROR { color:#ff7b72; }
    .lvl-WARN { color:#ffd33d; }
    .lvl-DEBUG { color:#79c0ff; }
  </style>
  <script>
    async function clearAll() {
      if (!confirm('Delete all messages?')) return;
      await fetch('/messages', { method: 'DELETE' });
      location.reload();
    }
    async function loadLogs() {
      const res = await fetch('/logs');
      const logs = await res.json();
      const el = document.getElementById('logs');
      el.innerHTML = logs.map(l => `\n<span class=\"lvl-${'${'}l.level}\">[${'${'}l.level}]</span> ${'${'}l.ts} — ${'${'}l.message}`).join('');
    }
    setInterval(loadLogs, 2000);
    window.addEventListener('load', loadLogs);
    async function doSearch() {
      const q = (document.getElementById('q')).value;
      const res = await fetch('/search?q=' + encodeURIComponent(q));
      const rows = await res.json();
      const tbody = document.getElementById('rows');
      tbody.innerHTML = rows.map(m => {
        const to = (m.to && m.to.length) ? m.to.join(', ') : '(none)';
        const subj = m.subject || '(no subject)';
        const from = m.from || '(unknown)';
        return `<tr><td><a href=\"/messages/${'${'}m.id}/html\">${'${'}m.id}</a></td><td>${'${'}m.received_at}</td><td>${'${'}from}</td><td>${'${'}to}</td><td>${'${'}subj}</td></tr>`;
      }).join('');
    }
  </script>
  </head>
<body>
  <h1>fauxmail</h1>
  <div class="actions">
    <button onclick="clearAll()">Clear All</button>
    <input id="q" placeholder="Search subject, from, text" onkeydown="if(event.key==='Enter')doSearch()" />
    <button onclick="doSearch()">Search</button>
  </div>
  <p>Send via REST: <code>POST /send</code> JSON {"to":["you@example.com"],"subject":"Hi"}</p>
  <table>
    <thead><tr><th>ID</th><th>Received</th><th>From</th><th>To</th><th>Subject</th></tr></thead>
    <tbody id="rows">__ROWS__</tbody>
  </table>
  <h2>Logs</h2>
  <div id="logs" class="logs" aria-live="polite"></div>
</body>
</html>
"#;
  Html(template.replace("__ROWS__", &rows))
}
