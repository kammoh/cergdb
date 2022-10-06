-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
    email       TEXT        NOT NULL UNIQUE PRIMARY KEY,
    password    TEXT        NOT NULL,
    name        TEXT,
    is_admin    BOOLEAN     NOT NULL
);              