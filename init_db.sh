#!/bin/bash

set -x
set -eo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
CARGO_MANIFEST_DIR="${CARGO_MANIFEST_DIR:-$SCRIPT_DIR}"

source .env
export PGPASSWORD="${DB_PASSWORD}"
export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed."
    echo "Trying to sqlx-cli install using cargo..."
    cargo install sqlx-cli --no-default-features --features postgres,rustls
fi

until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "${DB_USER}" -c '\q'; do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
done

sqlx database create

# sqlx migrate run
# cargo sqlx prepare --database-url ${DATABASE_URL}

>&2 echo "Database is now ready!"