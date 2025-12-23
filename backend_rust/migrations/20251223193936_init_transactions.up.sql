CREATE TYPE transaction_type AS ENUM (
    'deposit',
    'purchase',
    'withdrawal',
    'referral_payout',
    'service_charge',
    'refund'
);

CREATE TABLE transactions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT,
    order_id BIGINT,

    type transaction_type NOT NULL,
    amount NUMERIC(12,2) NOT NULL,

    store_balance_delta NUMERIC(12,2) NOT NULL,

    user_balance_after NUMERIC(12,2),
    store_balance_after NUMERIC(12,2) NOT NULL,

    platform_commission NUMERIC(12,2) NOT NULL DEFAULT 0.00,
    gateway_commission NUMERIC(12,2) NOT NULL DEFAULT 0.00,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    description TEXT,
    payment_gateway TEXT,
    details JSONB,

    CONSTRAINT fk_transactions_user
        FOREIGN KEY (user_id) REFERENCES bot_users(id) ON DELETE SET NULL,
    CONSTRAINT fk_transactions_order
        FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE SET NULL,

    CONSTRAINT chk_user_balance_requires_user
        CHECK (user_id IS NOT NULL = (user_balance_after IS NOT NULL))
);

CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions (user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_order_id ON transactions (order_id);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions (type);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_user_last_balance
    ON transactions (user_id, id DESC) INCLUDE (user_balance_after)
    WHERE user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_transactions_store_last_balance
    ON transactions (id DESC) INCLUDE (store_balance_after);

CREATE OR REPLACE FUNCTION calculate_balances()
RETURNS TRIGGER AS $$
DECLARE
    last_user_balance NUMERIC(12,2) := 0;
    last_store_balance NUMERIC(12,2) := 0;
BEGIN
    IF NEW.user_id IS NOT NULL THEN
        SELECT COALESCE(user_balance_after, 0)
        INTO last_user_balance
        FROM transactions
        WHERE user_id = NEW.user_id
        ORDER BY id DESC
        LIMIT 1;
        NEW.user_balance_after := last_user_balance + NEW.amount;
    END IF;

    SELECT COALESCE(store_balance_after, 0)
    INTO last_store_balance
    FROM transactions
    ORDER BY id DESC
    LIMIT 1;
    NEW.store_balance_after := last_store_balance + NEW.store_balance_delta;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_calculate_balances
    BEFORE INSERT ON transactions
    FOR EACH ROW
    EXECUTE FUNCTION calculate_balances();

CREATE OR REPLACE FUNCTION update_bot_user_balance()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.user_id IS NOT NULL THEN
        UPDATE bot_users
        SET 
            balance = NEW.user_balance_after,
            updated_at = NOW()
        WHERE id = NEW.user_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_bot_user_balance
    AFTER INSERT ON transactions
    FOR EACH ROW
    EXECUTE FUNCTION update_bot_user_balance();