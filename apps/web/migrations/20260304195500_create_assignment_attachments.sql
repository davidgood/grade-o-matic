CREATE TABLE assignment_attachments
(
    assignment_id uuid        NOT NULL,
    file_id       uuid        NOT NULL,
    created_by    uuid,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (assignment_id, file_id),

    FOREIGN KEY (assignment_id)
        REFERENCES assignments (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE,

    FOREIGN KEY (file_id)
        REFERENCES uploaded_files (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE,

    FOREIGN KEY (created_by)
        REFERENCES users (id)
        ON UPDATE CASCADE
        ON DELETE SET NULL
);

CREATE INDEX idx_assignment_attachments_assignment_id ON assignment_attachments (assignment_id);
CREATE INDEX idx_assignment_attachments_file_id ON assignment_attachments (file_id);
