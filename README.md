# CERG Benchmarking Database

## Create `.env`

```
RUST_LOG=actix_web=debug,info
TEST_LOG=enabled
RUST_BACKTRACE=1

SERVER_IP="0.0.0.0"
SERVER_PORT=4000


# database
DB_HOST=localhost
DB_USER=postgres
DB_NAME=cergdb
DB_PORT=5432
# DB_PASSWORD=xxx
DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

## path to certificates
TLS_CERT_PEM=${CARGO_MANIFEST_DIR}/certs/cert.pem
TLS_KEY_PEM=${CARGO_MANIFEST_DIR}/certs/key.pem
TLS=true
```

## Initialize Secret

```
$ ./init_secret.sh
```

## Database Initialization

```
$ ./init_db.sh
```

## Backup Database
```
$ ./backup_db.sh
```
