-- Add migration script here
ALTER TABLE classes
    ADD COLUMN points SMALLINT;
