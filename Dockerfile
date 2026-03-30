FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY target/release/backend /app/backend
COPY frontend/dist /app/frontend/dist

EXPOSE 8080

CMD ["/app/backend"]
