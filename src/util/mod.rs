//! Utility functions: tracing, HTML escape, mail parsing.

use mailparse::{MailHeaderMap, ParsedMail};
use tracing_subscriber::{EnvFilter, fmt};

/// Initialize pretty CLI logging.
pub fn init_tracing() {
  let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
  fmt()
    .with_env_filter(filter)
    .with_target(false)
    .pretty()
    .init();
}

/// Minimal HTML escaping for text display.
pub fn html_escape(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
}

/// Collect headers into a lowercase HashMap.
pub fn collect_headers(parsed: &ParsedMail<'_>) -> std::collections::HashMap<String, String> {
  let mut map = std::collections::HashMap::new();
  for h in &parsed.headers {
    map.insert(h.get_key().to_ascii_lowercase(), h.get_value());
  }
  map
}

/// Extract first text and HTML bodies from a MIME tree.
pub fn extract_bodies(parsed: &ParsedMail<'_>) -> (Option<String>, Option<String>) {
  if parsed.subparts.is_empty() {
    let ctype = parsed.ctype.mimetype.as_str();
    let data = parsed.get_body().unwrap_or_default();
    match ctype {
      "text/html" => (None, Some(data)),
      _ => (Some(data), None),
    }
  } else {
    let mut text = None;
    let mut html = None;
    for part in &parsed.subparts {
      let (t, h) = extract_bodies(part);
      if text.is_none() && t.is_some() {
        text = t;
      }
      if html.is_none() && h.is_some() {
        html = h;
      }
    }
    (text, html)
  }
}

/// Traverse MIME parts and collect attachment candidates.
pub fn collect_attachments(
  parsed: &ParsedMail<'_>,
  out: &mut Vec<(Option<String>, String, Vec<u8>)>,
) {
  if parsed.subparts.is_empty() {
    let ctype = parsed.ctype.mimetype.clone();
    let disp = parsed
      .headers
      .get_first_value("Content-Disposition")
      .unwrap_or_default();
    let mut filename: Option<String> = None;
    if let Some(pos) = disp.to_lowercase().find("filename=") {
      let part = &disp[pos + 9..];
      let cleaned = part
        .trim()
        .trim_matches(['"', '\''])
        .split(';')
        .next()
        .unwrap_or("");
      if !cleaned.is_empty() {
        filename = Some(cleaned.to_string());
      }
    }
    for (k, v) in &parsed.ctype.params {
      if k.eq_ignore_ascii_case("name") {
        filename.get_or_insert(v.clone());
      }
    }
    let is_text = ctype == "text/plain" || ctype == "text/html";
    let looks_attachment =
      disp.to_ascii_lowercase().contains("attachment") || filename.is_some() || (!is_text);
    if looks_attachment {
      let data = parsed.get_body_raw().unwrap_or_default();
      out.push((filename, parsed.ctype.mimetype.clone(), data));
    }
  } else {
    for part in &parsed.subparts {
      collect_attachments(part, out);
    }
  }
}
