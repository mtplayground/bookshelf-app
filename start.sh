#!/bin/sh
set -eu

mkdir -p /app/data

if [ -z "${DATABASE_URL:-}" ]; then
  export DATABASE_URL="sqlite:/app/data/bookshelf.db?mode=rwc"
fi

DB_FILE="/app/data/bookshelf.db"
if [ ! -f "$DB_FILE" ]; then
  sqlite3 "$DB_FILE" "PRAGMA journal_mode=WAL;" >/dev/null 2>&1 || true
fi

exec /app/backend
