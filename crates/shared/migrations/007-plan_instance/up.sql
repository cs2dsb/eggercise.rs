-- A plan instance is an actual execution of a plan on a given start_date.
-- Local to a certain user. Can be multiple instances of the same plan
CREATE TABLE plan_instance (
    id                  TEXT PRIMARY KEY,
    plan_id             TEXT NOT NULL,
    user_id             TEXT NOT NULL,

    start_date          TEXT,

    creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (plan_id) REFERENCES plan(id),
    FOREIGN KEY (user_id) REFERENCES user(id)
) STRICT;