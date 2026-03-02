-- Add migration script here
ALTER TABLE classes
    ADD COLUMN owner_id uuid;
ALTER TABLE classes
    ADD COLUMN term VARCHAR(255);
ALTER TABLE classes
    ADD CONSTRAINT fk_classes_owner_id
        FOREIGN KEY (owner_id)
            REFERENCES users (id)
            ON UPDATE CASCADE
            ON DELETE SET NULL;

CREATE INDEX idx_classes_owner_id ON classes(owner_id);
