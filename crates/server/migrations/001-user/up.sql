CREATE TABLE user (
    id                  TEXT PRIMARY KEY,
    username            TEXT NOT NULL UNIQUE,
    email               TEXT,
    display_name        TEXT,
    registration_date   TEXT DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT DEFAULT CURRENT_TIMESTAMP,
    last_login_date     TEXT
) STRICT;