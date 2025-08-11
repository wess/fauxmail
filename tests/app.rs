use axum::Router;
use fauxmail::{app::AppState, db, http};
use serde_json::json;
use sqlx::sqlite::SqlitePoolOptions;
use tokio::task::JoinHandle;

async fn start_server() -> (String, JoinHandle<()>) {
    let db_url = "sqlite://:memory:";
    let db_url = db::ensure_sqlite_path(db_url);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("connect memory sqlite");
    db::run_migrations(&pool).await.expect("migrate");
    let state = AppState { db: pool };
    let app: Router = http::build_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{}", addr), handle)
}

#[tokio::test]
async fn send_json_and_list() {
    let (base, _srv) = start_server().await;

    // Send via JSON
    let payload = json!({
        "from": "dev@example.test",
        "to": ["you@example.test"],
        "subject": "Hello JSON",
        "text": "Hi",
    });
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/send", base))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());
    let v: serde_json::Value = res.json().await.unwrap();
    let id = v.get("id").and_then(|x| x.as_str()).unwrap().to_string();

    // List messages
    let res = client
        .get(format!("{}/messages", base))
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());
    let arr: serde_json::Value = res.json().await.unwrap();
    assert!(
        arr.as_array()
            .unwrap()
            .iter()
            .any(|m| m["id"].as_str() == Some(&id))
    );
}

#[tokio::test]
async fn send_raw_and_fetch_html_and_attachments() {
    let (base, _srv) = start_server().await;

    // Minimal multipart with one attachment
    let eml = concat!(
        "From: dev@example.test\r\n",
        "To: you@example.test\r\n",
        "Subject: Hello Raw\r\n",
        "MIME-Version: 1.0\r\n",
        "Content-Type: multipart/mixed; boundary=BOUND\r\n",
        "\r\n",
        "--BOUND\r\n",
        "Content-Type: text/plain\r\n\r\n",
        "Hi text\r\n",
        "--BOUND\r\n",
        "Content-Type: application/octet-stream\r\n",
        "Content-Disposition: attachment; filename=\"a.txt\"\r\n\r\n",
        "ABC123\r\n",
        "--BOUND--\r\n",
    );
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/send/raw", base))
        .body(eml.as_bytes().to_vec())
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());
    let v: serde_json::Value = res.json().await.unwrap();
    let id = v["id"].as_str().unwrap().to_string();

    // Fetch HTML view
    let res = client
        .get(format!("{}/messages/{}/html", base, id))
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());
    let html = res.text().await.unwrap();
    assert!(html.contains("Hello Raw"));

    // List attachments
    let res = client
        .get(format!("{}/messages/{}/attachments", base, id))
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());
    let atts: serde_json::Value = res.json().await.unwrap();
    let arr = atts.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    let att_id = arr[0]["id"].as_str().unwrap();

    // Download attachment
    let res = client
        .get(format!("{}/attachments/{}/download", base, att_id))
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());
    let body = res.bytes().await.unwrap();
    assert_eq!(&body[..], b"ABC123");
}

#[tokio::test]
async fn search_endpoint_filters_results() {
    let (base, _srv) = start_server().await;
    let client = reqwest::Client::new();

    // Insert two messages via JSON
    for subj in ["Alpha One", "Beta Two"] {
        let payload = json!({ "to": ["you@example.test"], "subject": subj, "text": "x" });
        let res = client
            .post(format!("{}/send", base))
            .json(&payload)
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success());
    }

    // Search for 'Alpha'
    let res = client
        .get(format!("{}/search?q=Alpha", base))
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());
    let arr: serde_json::Value = res.json().await.unwrap();
    let arr = arr.as_array().unwrap();
    assert!(
        arr.iter()
            .all(|m| m["subject"].as_str().unwrap().contains("Alpha"))
    );
}

#[tokio::test]
async fn clear_messages_and_logs_entry() {
    let (base, _srv) = start_server().await;
    let client = reqwest::Client::new();

    // Create two messages
    for subj in ["One", "Two"] {
        let payload = json!({ "to": ["you@example.test"], "subject": subj });
        let res = client
            .post(format!("{}/send", base))
            .json(&payload)
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success());
    }

    // Clear all
    let res = client
        .delete(format!("{}/messages", base))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::NO_CONTENT);

    // Now list should be empty
    let res = client
        .get(format!("{}/messages", base))
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());
    let arr: serde_json::Value = res.json().await.unwrap();
    assert_eq!(arr.as_array().unwrap().len(), 0);

    // Logs contain an entry about clearing
    let res = client.get(format!("{}/logs", base)).send().await.unwrap();
    assert!(res.status().is_success());
    let logs: serde_json::Value = res.json().await.unwrap();
    let found = logs.as_array().unwrap().iter().any(|l| {
        l["message"]
            .as_str()
            .unwrap_or("")
            .contains("cleared all messages")
    });
    assert!(found, "expected a log entry about clearing messages");
}

#[tokio::test]
async fn logs_feed_contains_send_event() {
    let (base, _srv) = start_server().await;
    let client = reqwest::Client::new();

    let payload = json!({ "to": ["you@example.test"], "subject": "Log Me" });
    let res = client
        .post(format!("{}/send", base))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());

    let res = client.get(format!("{}/logs", base)).send().await.unwrap();
    assert!(res.status().is_success());
    let logs: serde_json::Value = res.json().await.unwrap();
    let found = logs.as_array().unwrap().iter().any(|l| {
        l["message"]
            .as_str()
            .unwrap_or("")
            .contains("stored message via REST")
    });
    assert!(found, "expected a REST stored log entry");
}
