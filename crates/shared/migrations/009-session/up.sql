CREATE TABLE session (
    id                  TEXT PRIMARY KEY,
    plan_instance_id    TEXT NOT NULL,

    planned_date        TEXT NOT NULL,
    performed_date      TEXT,
    
    creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (plan_instance_id) REFERENCES plan_instance(id)
) STRICT;