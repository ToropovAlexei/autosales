CREATE TABLE transactions (
    id BIGSERIAL PRIMARY KEY,
    customer_id BIGINT,
    order_id BIGINT,

    type TEXT NOT NULL CHECK (type IN ('deposit', 'purchase', 'withdrawal', 'referral_payout', 'service_charge', 'refund')),
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
        FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE SET NULL,
    CONSTRAINT fk_transactions_order
        FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE SET NULL,

    CONSTRAINT chk_user_balance_requires_user
        CHECK (customer_id IS NOT NULL = (user_balance_after IS NOT NULL))
);

CREATE INDEX IF NOT EXISTS idx_transactions_customer_id ON transactions (customer_id);
CREATE INDEX IF NOT EXISTS idx_transactions_order_id ON transactions (order_id);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions (type);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_user_last_balance
    ON transactions (customer_id, id DESC) INCLUDE (user_balance_after)
    WHERE customer_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_transactions_store_last_balance
    ON transactions (id DESC) INCLUDE (store_balance_after);

CREATE OR REPLACE FUNCTION calculate_balances()
RETURNS TRIGGER AS $$
DECLARE
    last_user_balance NUMERIC(12,2);
    last_store_balance NUMERIC(12,2);
BEGIN
    IF NEW.customer_id IS NOT NULL THEN
        SELECT COALESCE((SELECT user_balance_after
                        FROM transactions
                        WHERE customer_id = NEW.customer_id
                        ORDER BY id DESC
                        LIMIT 1), 0)
        INTO last_user_balance;
        NEW.user_balance_after := last_user_balance + NEW.amount;
    END IF;

    SELECT COALESCE((SELECT store_balance_after
                    FROM transactions
                    ORDER BY id DESC
                    LIMIT 1), 0)
    INTO last_store_balance;
    NEW.store_balance_after := last_store_balance + NEW.store_balance_delta;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_calculate_balances
    BEFORE INSERT ON transactions
    FOR EACH ROW
    EXECUTE FUNCTION calculate_balances();

CREATE OR REPLACE FUNCTION update_customer_balance()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.customer_id IS NOT NULL THEN
        UPDATE customers
        SET 
            balance = NEW.user_balance_after,
            updated_at = NOW()
        WHERE id = NEW.customer_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_customer_balance
    AFTER INSERT ON transactions
    FOR EACH ROW
    EXECUTE FUNCTION update_customer_balance();