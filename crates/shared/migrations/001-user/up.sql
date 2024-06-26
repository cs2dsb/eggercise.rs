CREATE TABLE user (
    id                              TEXT PRIMARY KEY,

    username                        TEXT NOT NULL UNIQUE,
    email                           TEXT,
    display_name                    TEXT,
    push_notification_subscription  TEXT,

    creation_date                   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date               TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login_date                 TEXT
) STRICT;