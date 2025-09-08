# ---------- Build stage ----------
FROM rust:1.87-bookworm AS builder
WORKDIR /app

# Copy everything and build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# ---------- Runtime stage ----------
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*
RUN useradd -u 10001 -m appuser

WORKDIR /app
COPY --from=builder /app/target/release/ai-adapter /app/ai-adapter

ENV RUST_LOG=info APP_HOST=0.0.0.0 APP_PORT=8080
EXPOSE 8080
USER appuser
CMD ["/app/ai-adapter"]
