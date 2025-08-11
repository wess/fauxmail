use lettre::message::{Message, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::response::Response;
use lettre::SmtpTransport;

fn main() {
    let host = std::env::var("FAUXMAIL_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("FAUXMAIL_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(1025);
    let user = std::env::var("FAUXMAIL_SMTP_USER").ok();
    let pass = std::env::var("FAUXMAIL_SMTP_PASS").ok();

    let mut builder = SmtpTransport::builder_dangerous(host).port(port);
    if let (Some(u), Some(p)) = (user, pass) {
        builder = builder.credentials(Credentials::new(u, p));
    }
    let mailer = builder.build();

    let email = Message::builder()
        .from("dev@example.test".parse().unwrap())
        .to("you@example.test".parse().unwrap())
        .subject("Hello from Rust")
        .singlepart(SinglePart::plain("Hi from lettre via fauxmail")).unwrap();

    let resp: Response = mailer.send(&email).expect("send failed");
    println!("Sent: {}", resp.code());
}

