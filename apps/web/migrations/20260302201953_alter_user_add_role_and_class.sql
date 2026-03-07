-- Add migration script here
ALTER TABLE users
    ADD COLUMN user_role VARCHAR(32) NOT NULL DEFAULT 'student';

ALTER TABLE users
    ADD CONSTRAINT chk_users_user_role
        CHECK (user_role IN ('admin', 'instructor', 'ta', 'student'));

CREATE TABLE class_memberships (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    class_id uuid NOT NULL,
    user_id uuid NOT NULL,
    role VARCHAR(16) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (class_id, user_id),
    CONSTRAINT fk_class_memberships_class_id
        FOREIGN KEY (class_id)
            REFERENCES classes (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    CONSTRAINT fk_class_memberships_user_id
        FOREIGN KEY (user_id)
            REFERENCES users (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    CONSTRAINT chk_class_memberships_role
        CHECK (role IN ('ta', 'student'))
);

CREATE INDEX idx_class_memberships_user_id ON class_memberships(user_id);
CREATE INDEX idx_class_memberships_class_id ON class_memberships(class_id);
