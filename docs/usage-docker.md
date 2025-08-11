# Using fauxmail with Docker

## Build the image

- From repo root:
  - `docker build -t fauxmail:local .`

## Run the container

- Default ports and DB under `/data`:
  - `docker run --rm -p 8025:8025 -p 1025:1025 -v "$PWD/data":/data fauxmail:local`
- With SMTP AUTH:
  - `docker run --rm -p 8025:8025 -p 1025:1025 \
    -e FAUXMAIL_SMTP_USER=dev -e FAUXMAIL_SMTP_PASS=secret \
    -e FAUXMAIL_DATABASE=sqlite:///data/fauxmail.db \
    -v "$PWD/data":/data fauxmail:local`

## Verify

- Open `http://localhost:8025/` for the dashboard.
- Send a test email using SMTP examples in `examples/`.

