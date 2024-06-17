CREATE TABLE exercise_group (
    id                  TEXT PRIMARY KEY,

    name                TEXT NOT NULL,
    description         TEXT,

    creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
) STRICT;

CREATE TABLE exercise_group_member (
    id                  TEXT PRIMARY KEY,
    exercise_id         TEXT NOT NULL,
    group_id            TEXT NOT NULL,

    FOREIGN KEY (exercise_id) REFERENCES exercise(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES exercise_group(id) ON DELETE CASCADE
) STRICT;