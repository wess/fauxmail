# SMTP Authentication

fauxmail supports AUTH PLAIN and AUTH LOGIN to mimic common providers.

## Configure credentials

- Set env vars:
  - `FAUXMAIL_SMTP_USER=dev`
  - `FAUXMAIL_SMTP_PASS=secret`
- If not set, SMTP accepts mail without authentication.

## Client expectations

- Host/port: `127.0.0.1:1025`
- TLS: none (plain TCP)
- AUTH methods: PLAIN, LOGIN

See `examples/` for language-specific SMTP client snippets.

