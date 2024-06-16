CREATE TABLE plan (
    id                  TEXT PRIMARY KEY,
    owner_id            TEXT NOT NULL,
    
    name                TEXT NOT NULL UNIQUE,
    description         TEXT,
    
    creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (owner_id) REFERENCES user(id)
) STRICT;

CREATE TABLE plan_instance (
    id                  TEXT PRIMARY KEY,
    user_id             TEXT NOT NULL,

    creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (user_id) REFERENCES user(id)
) STRICT;

CREATE TABLE plan_exercise_group (
    id                  TEXT PRIMARY KEY,
    plan_id             TEXT NOT NULL,
    exercise_group_id   TEXT NOT NULL,
    
    notes               TEXT,
    
    creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (plan_id) REFERENCES plan(id),
    FOREIGN KEY (exercise_group_id) REFERENCES exercise_group(id)
) STRICT;