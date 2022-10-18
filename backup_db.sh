#!/bin/bash

set -x
set -eo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

source .env
export PGPASSWORD="${DB_PASSWORD}"
export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

TIMESTAMP=$(date +%d-%m-%Y_%H-%M-%S)

echo ${DB_PASSWORD} >.pgpass

if [[ -z ${FORMAT} ]]; then
    FORMAT="tar"
fi

case ${FORMAT} in
tar)
    EXT=".tar"
    PG_DUMP_ARGS+=" -b "
    ;;
plain)
    EXT=".sql"
    ;;
*)
    echo >&2 "Invalid FORMAT"
    exit 1
    ;;
esac

BACKUP_FILENAME=${DB_NAME}_backup_${TIMESTAMP}${EXT}.gz

pg_dump --username ${DB_USER} --host=${DB_HOST} --port=${DB_PORT} -F ${FORMAT} ${PG_DUMP_ARGS} ${DB_NAME} | gzip >${BACKUP_FILENAME}
rm -f .pgpass

echo >&2 backup of ${DB_NAME} saved to ${BACKUP_FILENAME}
