# ── Build stage ──────────────────────────────────────────────
FROM rust:1.88-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependency compilation: copy manifests first, build a dummy,
# then copy the real source and rebuild only the application crate.
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

COPY src ./src
# Touch main.rs so cargo detects the source change
RUN touch src/main.rs
RUN cargo build --release

# ── Runtime stage ────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --system appuser && useradd --system --gid appuser appuser

WORKDIR /app

COPY --from=builder /app/target/release/atlas-backend ./atlas-backend

RUN mkdir -p /app/uploads && chown appuser:appuser /app/uploads

USER appuser

ENV HOST=0.0.0.0
EXPOSE 8080

CMD ["./atlas-backend"]
