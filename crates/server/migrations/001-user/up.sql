CREATE TABLE user (
    id              INTEGER PRIMARY KEY,
    name            TEXT NOT NULL,
    first_login     TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    latest_login    TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
) STRICT;