CREATE TABLE credential (
    id                  TEXT PRIMARY KEY,
    user_id             TEXT NOT NULL,

    passkey             TEXT NOT NULL,
    counter             INTEGER NOT NULL DEFAULT 0,
    backup_eligible     INTEGER NOT NULL DEFAULT 0,
    backup_state        INTEGER NOT NULL DEFAULT 0,

    creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_date      TEXT,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX idx_credential_user_id
ON credential(user_id);