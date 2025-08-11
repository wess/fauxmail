# Using fauxmail Locally

## Start the server

- Default (HTTP 8025, SMTP 1025, SQLite `./fauxmail.db`):
  - `cargo run`
- Custom ports and DB:
  - `FAUXMAIL_ADDR=127.0.0.1:8900 FAUXMAIL_SMTP_ADDR=127.0.0.1:2525 FAUXMAIL_DATABASE=sqlite:///tmp/fauxmail.db cargo run`

## SMTP auth (optional)

- Set credentials to require AUTH (PLAIN/LOGIN supported):
  - `FAUXMAIL_SMTP_USER=dev FAUXMAIL_SMTP_PASS=secret cargo run`
- Without these variables, SMTP accepts messages without AUTH.

## Dashboard

- Open `http://127.0.0.1:8025/` to view messages and logs.

## REST endpoints

- Send JSON: `POST /send` with `{from?, to[], subject?, text?, html?, headers?}`
- Send EML: `POST /send/raw` with raw RFC822 content
- List: `GET /messages`, `GET /messages/:id`, `GET /messages/:id/html`
- Clear: `DELETE /messages`
- Logs: `GET /logs`

