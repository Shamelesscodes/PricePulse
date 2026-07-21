# Build Stage
FROM rust:bookworm as builder

WORKDIR /usr/src/pricepulse

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy source code and manifest
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# Build production release binary
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

WORKDIR /app

# Install SSL certificates & curl (required for scraping & HTTPS calls)
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Copy release binary and configuration from builder
COPY --from=builder /usr/src/pricepulse/target/release/pricepulse /app/pricepulse
COPY config /app/config

# Expose API Port
EXPOSE 3000

# Run API server & background scheduler by default
CMD ["/app/pricepulse", "serve"]
