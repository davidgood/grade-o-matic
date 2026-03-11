CREATE TABLE grading_jobs
(
    id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id uuid        NOT NULL,
    file_id       uuid        NOT NULL,
    submitted_by  uuid,
    status        VARCHAR(16) NOT NULL DEFAULT 'queued',
    attempt_count INTEGER     NOT NULL DEFAULT 0,
    locked_at     TIMESTAMPTZ,
    started_at    TIMESTAMPTZ,
    completed_at  TIMESTAMPTZ,
    error_message TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_grading_jobs_assignment_id
        FOREIGN KEY (assignment_id)
            REFERENCES assignments (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,

    CONSTRAINT fk_grading_jobs_file_id
        FOREIGN KEY (file_id)
            REFERENCES uploaded_files (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,

    CONSTRAINT fk_grading_jobs_submitted_by
        FOREIGN KEY (submitted_by)
            REFERENCES users (id)
            ON UPDATE CASCADE
            ON DELETE SET NULL,

    CONSTRAINT uq_grading_jobs_assignment_file
        UNIQUE (assignment_id, file_id),

    CONSTRAINT chk_grading_jobs_status
        CHECK (status IN ('queued', 'running', 'completed', 'failed'))
);

CREATE INDEX idx_grading_jobs_status_created_at
    ON grading_jobs (status, created_at);
