# CERG Benchmarking Database


## Setup PostgreSQL
install postgresql
set password
```
sudo -u postgres psql postgres

# \password postgres

Enter new password: 
\q
```

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
DB_PASSWORD=...
DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

## path to certificates
TLS_CERT_PEM=...
TLS_KEY_PEM=...
TLS=true
```


## Initialize Secret

```
$ ./init_secret.sh
```

## Initialization Database

```
$ ./init_db.sh
```


## Backup Database
```
$ ./backup_db.sh
```

## Delete and Reset the Database
```
$ sqlx database reset
```
