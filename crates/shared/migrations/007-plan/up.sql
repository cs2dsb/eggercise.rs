-- A plan is the a methodology for the exercise programme. Global for all users
CREATE TABLE plan (
    id                  TEXT PRIMARY KEY,
    owner_id            TEXT NOT NULL,
    
    name                TEXT NOT NULL UNIQUE,
    description         TEXT,
    duration_weeks      INTEGER NOT NULL,
    
    creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (owner_id) REFERENCES user(id)
) STRICT;