ALTER TABLE bots
    ADD COLUMN deleted_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_bots_deleted_at
    ON bots (deleted_at)
    WHERE deleted_at IS NOT NULL;
