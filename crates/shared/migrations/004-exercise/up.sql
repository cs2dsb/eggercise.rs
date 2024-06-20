CREATE TABLE exercise (
    id                  TEXT PRIMARY KEY,

    name                TEXT NOT NULL UNIQUE,
    description         TEXT,
    base_recovery_days  REAL NOT NULL DEFAULT 3.5, 
    
    creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
) STRICT;