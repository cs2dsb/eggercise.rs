CREATE TABLE service_version (
    id              TEXT PRIMARY KEY,
    version         TEXT NOT NULL UNIQUE,
    creation_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
) STRICT;