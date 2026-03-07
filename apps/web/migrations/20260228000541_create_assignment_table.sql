-- Add migration script here
CREATE TABLE classes
(
    id          uuid PRIMARY KEY  DEFAULT gen_random_uuid(),
    title       VARCHAR(255) NOT NULL,
    description TEXT,
    created_by  uuid,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_by uuid,
    modified_at TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (created_by)
        REFERENCES users (id)
        ON UPDATE CASCADE
        ON DELETE SET NULL,

    FOREIGN KEY (modified_by)
        REFERENCES users (id)
        ON UPDATE CASCADE
        ON DELETE SET NULL
);

CREATE TABLE assignments
(
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    class_id    uuid NOT NULL,
    title       VARCHAR(255) NOT NULL,
    description TEXT,
    due_at      TIMESTAMPTZ,
    created_by  uuid,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_by uuid,
    modified_at TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (created_by)
        REFERENCES users (id)
        ON UPDATE CASCADE
        ON DELETE SET NULL,

    FOREIGN KEY (modified_by)
        REFERENCES users (id)
        ON UPDATE CASCADE
        ON DELETE SET NULL,

    FOREIGN KEY (class_id)
         REFERENCES classes (id)
         ON UPDATE CASCADE
         ON DELETE CASCADE

);

CREATE INDEX idx_assignments_class_id ON assignments(class_id);
