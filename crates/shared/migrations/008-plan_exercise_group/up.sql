-- A plan exercise group is a sub group of exercises that are configured on 
-- a given plan. This is the level at which progression is programmed. Group
-- can contain one or many exercises. Plan can contain one or more groups
CREATE TABLE plan_exercise_group (
    id                  TEXT PRIMARY KEY,
    plan_id             TEXT NOT NULL,
    exercise_group_id   TEXT NOT NULL,
    
    notes               TEXT,
    config              TEXT,
    
    creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (plan_id) REFERENCES plan(id),
    FOREIGN KEY (exercise_group_id) REFERENCES exercise_group(id)
) STRICT;