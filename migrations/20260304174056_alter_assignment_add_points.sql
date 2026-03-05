-- Add migration script here
ALTER TABLE assignments
    ADD COLUMN points SMALLINT;
