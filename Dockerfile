# ---------- Chef base ----------
FROM rust:1.87-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

# ---------- Plan (calc dep graph) ----------
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---------- Cook deps ----------
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# ---------- Build app ----------
COPY . .
RUN cargo build --release

# ---------- Runtime ----------
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
