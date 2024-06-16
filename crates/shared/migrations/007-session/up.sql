CREATE TABLE session (
    id                  TEXT PRIMARY KEY,
    creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
) STRICT;