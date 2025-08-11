# fauxmail

A lightweight, local-only mock email server for development. It provides:

- A simple dashboard to view captured messages.
- A REST API to "send" emails (stored, not delivered).
- An endpoint to POST raw EML and parse MIME parts.
 - A simple SMTP listener for capturing messages on `127.0.0.1:1025`.

## Quickstart

- Run the server: `cargo run`
- Open the dashboard: `http://127.0.0.1:8025/`
- Change bind address with `FAUXMAIL_ADDR` (e.g., `FAUXMAIL_ADDR=0.0.0.0:8025`).
 - Change SMTP address with `FAUXMAIL_SMTP_ADDR` (e.g., `FAUXMAIL_SMTP_ADDR=127.0.0.1:1025`).
 - Use SQLite file with `FAUXMAIL_DATABASE` (e.g., `sqlite:///tmp/fauxmail.db`).

## Send via REST (JSON)

```
curl -X POST http://127.0.0.1:8025/send \
  -H 'content-type: application/json' \
  -d '{
    "from":"dev@example.test",
    "to":["you@example.test"],
    "subject":"Hello",
    "text":"Hi from fauxmail"
  }'
```

## Send raw EML

```
curl -X POST http://127.0.0.1:8025/send/raw --data-binary @message.eml
```

## Send via SMTP

Use any SMTP client, pointing at `127.0.0.1:1025` without TLS/auth. Example with `swaks`:

```
swaks --server 127.0.0.1:1025 \
  --from dev@example.test --to you@example.test \
  --data 'Subject: Hello\n\nHello from SMTP'
```

## API

- `GET /messages`: JSON list of messages
- `GET /messages/:id`: JSON single message
- `GET /messages/:id/html`: Rendered HTML view
- `DELETE /messages`: Clear all messages
- `POST /send`: Accepts JSON {from?, to[], subject?, text?, html?, headers?}
- `POST /send/raw`: Accepts raw RFC822/EML; parses text/html parts

Logs are printed to the CLI and streamed to the dashboard Logs panel. Set verbosity with `RUST_LOG` (e.g., `RUST_LOG=debug`).

## License

MIT — see `LICENSE` for details.
