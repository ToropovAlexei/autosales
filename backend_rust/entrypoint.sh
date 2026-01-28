#!/bin/sh
set -e

if [ -z "${DATABASE_URL:-}" ]; then
  if [ -z "${DATABASE_USER:-}" ] || [ -z "${DATABASE_PASSWORD:-}" ] || [ -z "${DATABASE_HOST:-}" ] || [ -z "${DATABASE_PORT:-}" ] || [ -z "${DATABASE_NAME:-}" ]; then
    echo "DATABASE_URL or DATABASE_* env vars must be set" >&2
    exit 1
  fi

  export DATABASE_URL="postgres://${DATABASE_USER}:${DATABASE_PASSWORD}@${DATABASE_HOST}:${DATABASE_PORT}/${DATABASE_NAME}"
fi

sqlx migrate run --source /app/migrations
init

if [ "${SEED_DB:-}" = "1" ] || [ "${SEED_DB:-}" = "true" ]; then
  seed
fi

exec "$@"
