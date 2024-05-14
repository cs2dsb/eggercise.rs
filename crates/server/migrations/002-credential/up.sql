CREATE TABLE credential (
    id                  TEXT PRIMARY KEY,
    user_id             TEXT NOT NULL,
    passkey             TEXT NOT NULL,
    public_key          TEXT,
    -- attestation_type    TEXT NOT NULL,
    aaguid              TEXT DEFAULT '00000000-0000-0000-0000-000000000000',
    signature_count     INTEGER,
    creation_date       TEXT DEFAULT CURRENT_TIMESTAMP,
    last_used_date      TEXT,
    last_updated_date   TEXT DEFAULT CURRENT_TIMESTAMP,
    type_               TEXT,
    transports          TEXT,
    backup_eligible     INTEGER DEFAULT 0,
    backup_state        INTEGER DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX idx_credential_user_id
ON credential(user_id);