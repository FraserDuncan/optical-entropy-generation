# Multi-stage build for Optical Entropy Generator
FROM rust:1.75-bookworm AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY src/ src/

RUN cargo build --release --features metrics

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/optical-entropy /usr/local/bin/optical-entropy

ENV RUST_LOG=info
ENV METRICS_PORT=9090

EXPOSE 9090

ENTRYPOINT ["optical-entropy"]
