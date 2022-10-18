#!/bin/bash

set -x
set -eo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

source .env
export PGPASSWORD="${DB_PASSWORD}"
export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

TIMESTAMP=$(date +%d-%m-%Y_%H-%M-%S)
BACKUP_FILENAME=${DB_NAME}_backup_${TIMESTAMP}.tar

echo ${DB_PASSWORD}  > .pgpass

pg_dump --username ${DB_USER} --host=${DB_HOST} --port=${DB_PORT}  -b -F t ${DB_NAME} > ${BACKUP_FILENAME}

echo backup of ${DB_NAME} saved to ${BACKUP_FILENAME}
rm -f .pgpass
