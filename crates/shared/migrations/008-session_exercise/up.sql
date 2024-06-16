CREATE TABLE session_exercise (
    id                  TEXT PRIMARY KEY,
    exercise_id         TEXT NOT NULL,
    session_id          TEXT NOT NULL,
    creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (exercise_id) REFERENCES exercise(id),
    FOREIGN KEY (session_id) REFERENCES session(id) ON DELETE CASCADE
) STRICT;