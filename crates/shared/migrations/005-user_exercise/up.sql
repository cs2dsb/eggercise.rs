CREATE TABLE user_exercise (
    id                  TEXT PRIMARY KEY,
    exercise_id         TEXT NOT NULL,
    user_id             TEXT NOT NULL,

    recovery_days       REAL, 
    
    creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (exercise_id) REFERENCES exercise(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
) STRICT;