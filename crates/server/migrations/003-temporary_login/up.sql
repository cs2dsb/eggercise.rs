CREATE TABLE temporary_login (
    id                  TEXT PRIMARY KEY,
    user_id             TEXT NOT NULL,
    expiry_date         TEXT NOT NULL,
    url                 TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX idx_temporary_login_user_id
ON temporary_login(user_id);