# fauxmail

A lightweight, local-only mock email server for development. It provides:

- A simple dashboard to view captured messages.
- A REST API to "send" emails (stored, not delivered).
- An endpoint to POST raw EML and parse MIME parts.
 - A simple SMTP listener for capturing messages on `127.0.0.1:1025`.

## Quickstart

Install from GitHub Releases (recommended):

- Download the archive for your OS from Releases.
- macOS (Apple Silicon) / Linux:
  - `tar -xzf fauxmail-vX.Y.Z-<platform>.tar.gz`
  - `cd fauxmail-vX.Y.Z-<platform>`
  - Optionally move to PATH: `sudo mv fauxmail /usr/local/bin/`
- Windows:
  - Unzip `fauxmail-vX.Y.Z-windows-x86_64.zip`
  - Optionally add folder to PATH or move `fauxmail.exe` somewhere on PATH.

Run:

- Start: `fauxmail`
- Open: `http://127.0.0.1:8025/`
- Version: `fauxmail --version`

Config env vars:

- `FAUXMAIL_ADDR` (HTTP, default `127.0.0.1:8025`)
- `FAUXMAIL_SMTP_ADDR` (SMTP, default `127.0.0.1:1025`)
- `FAUXMAIL_DATABASE` (e.g., `sqlite://fauxmail.db` or `sqlite:///data/fauxmail.db`)
- `FAUXMAIL_SMTP_USER`, `FAUXMAIL_SMTP_PASS` (enable SMTP AUTH)

Linux portability: releases use a static musl build for broad compatibility.

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

## Docker

- Build local: `docker build -t fauxmail:local .`
- Run local: `docker run --rm -p 8025:8025 -p 1025:1025 -v "$PWD/data":/data fauxmail:local`
- With AUTH and persisted DB: add `-e FAUXMAIL_SMTP_USER=dev -e FAUXMAIL_SMTP_PASS=secret -e FAUXMAIL_DATABASE=sqlite:///data/fauxmail.db`
- Pull from GHCR (after first release): `docker pull ghcr.io/wess/fauxmail:latest`
- Run from GHCR: `docker run --rm -p 8025:8025 -p 1025:1025 ghcr.io/wess/fauxmail:latest`

## License

MIT — see `LICENSE` for details.

## Checksums

Releases include `SHA256SUMS.txt` for all artifacts:

- Linux/macOS: `sha256sum -c SHA256SUMS.txt` (or `shasum -a 256 -c ...`)
- Windows (PowerShell): `Get-FileHash fauxmail-*.zip -Algorithm SHA256`
