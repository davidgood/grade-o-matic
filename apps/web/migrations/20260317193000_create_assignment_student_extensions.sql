CREATE TABLE assignment_student_extensions
(
    assignment_id uuid        NOT NULL,
    student_id    uuid        NOT NULL,
    due_at        TIMESTAMPTZ NOT NULL,
    created_by    uuid,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_by   uuid,
    modified_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (assignment_id, student_id),

    FOREIGN KEY (assignment_id)
        REFERENCES assignments (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE,

    FOREIGN KEY (student_id)
        REFERENCES users (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE,

    FOREIGN KEY (created_by)
        REFERENCES users (id)
        ON UPDATE CASCADE
        ON DELETE SET NULL,

    FOREIGN KEY (modified_by)
        REFERENCES users (id)
        ON UPDATE CASCADE
        ON DELETE SET NULL
);

CREATE INDEX idx_assignment_student_extensions_assignment_id
    ON assignment_student_extensions (assignment_id);

CREATE INDEX idx_assignment_student_extensions_student_id
    ON assignment_student_extensions (student_id);
