ALTER TABLE transactions
    ADD COLUMN bot_id BIGINT;

ALTER TABLE transactions
    ADD CONSTRAINT fk_transactions_bot
        FOREIGN KEY (bot_id) REFERENCES bots(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_transactions_bot_id
    ON transactions (bot_id);
