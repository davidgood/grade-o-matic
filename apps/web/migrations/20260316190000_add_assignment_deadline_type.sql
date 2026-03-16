CREATE TYPE assignment_deadline_type_enum AS ENUM ('hard_cutoff', 'soft_deadline');

ALTER TABLE assignments
    ADD COLUMN deadline_type assignment_deadline_type_enum NOT NULL DEFAULT 'soft_deadline';
