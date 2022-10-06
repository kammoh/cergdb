-- Add up migration script here
CREATE TABLE IF NOT EXISTS results (
    id           TEXT        NOT NULL UNIQUE PRIMARY KEY,
    name         TEXT,
    timestamp    TIMESTAMPTZ NOT NULL,
    category     TEXT,
    metadata     JSONB,
    timing       JSONB,
    synthesis    JSONB
);
