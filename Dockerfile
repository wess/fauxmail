# syntax=docker/dockerfile:1

FROM rust:1.86 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:stable-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
ENV FAUXMAIL_ADDR=0.0.0.0:8025
ENV FAUXMAIL_SMTP_ADDR=0.0.0.0:1025
ENV FAUXMAIL_DATABASE=sqlite:///data/fauxmail.db
VOLUME ["/data"]
COPY --from=builder /app/target/release/fauxmail /usr/local/bin/fauxmail
EXPOSE 8025 1025
CMD ["/usr/local/bin/fauxmail"]

