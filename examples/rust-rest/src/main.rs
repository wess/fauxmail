use reqwest::blocking::Client;
use serde::Serialize;

#[derive(Serialize)]
struct SendReq<'a> {
    from: &'a str,
    to: Vec<&'a str>,
    subject: &'a str,
    text: &'a str,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("FAUXMAIL_HTTP").unwrap_or_else(|_| "http://127.0.0.1:8025/send".into());
    let req = SendReq {
        from: "dev@example.test",
        to: vec!["you@example.test"],
        subject: "Hello via REST (Rust)",
        text: "Hi from Rust using REST",
    };
    let client = Client::new();
    let resp = client.post(url).json(&req).send()?.text()?;
    println!("{}", resp);
    Ok(())
}

