#[tokio::main]
async fn main() {
  // Minimal CLI: support --version/-V
  let mut args = std::env::args().skip(1);
  if let Some(arg) = args.next() {
    if arg == "--version" || arg == "-V" {
      println!("fauxmail {}", env!("CARGO_PKG_VERSION"));
      return;
    }
    // Allow running without args; any other arg prints help
    if arg == "--help" || arg == "-h" {
      eprintln!("Usage: fauxmail [--version]");
      return;
    }
  }

  if let Err(e) = fauxmail::app::run().await {
    eprintln!("error: {e}");
    std::process::exit(1);
  }
}
