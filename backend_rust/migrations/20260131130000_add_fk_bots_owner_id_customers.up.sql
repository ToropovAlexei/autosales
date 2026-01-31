ALTER TABLE bots
    ADD CONSTRAINT fk_bots_owner_id
        FOREIGN KEY (owner_id) REFERENCES customers(id) ON DELETE SET NULL;