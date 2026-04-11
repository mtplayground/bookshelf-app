# Self-check: host `cargo build --release` did not succeed because cargo is not installed in this executor; using PATH B with cargo-chef.
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY backend backend
COPY frontend frontend
COPY types types
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependency graph first so app source edits do not invalidate dependency layers.
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk --locked
RUN trunk build --release frontend/index.html
RUN cargo build --release -p backend

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/frontend/dist /app/frontend/dist
COPY start.sh /app/start.sh
RUN chmod +x /app/start.sh && mkdir -p /app/data

ENV DATABASE_URL=sqlite:/app/data/bookshelf.db?mode=rwc
ENV RUST_LOG=info

EXPOSE 8080
CMD ["/app/start.sh"]
