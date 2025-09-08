# ---------- Build stage ----------
FROM rust:1.80-bookworm AS builder

WORKDIR /app

# 1) Cache deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# 2) Build the real binary
COPY src ./src
RUN cargo build --release

# ---------- Runtime stage ----------
FROM debian:bookworm-slim AS runtime

# certs for reqwest/rustls
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# non-root user
RUN useradd -u 10001 -m appuser

WORKDIR /app
COPY --from=builder /app/target/release/ai-adapter /app/ai-adapter

# sensible defaults (override via compose/env)
ENV RUST_LOG=info \
    APP_HOST=0.0.0.0 \
    APP_PORT=8080

EXPOSE 8080
USER appuser
CMD ["/app/ai-adapter"]
