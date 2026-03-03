CREATE TABLE customers (
    id BIGSERIAL PRIMARY KEY,
    telegram_id BIGINT NOT NULL UNIQUE,
    balance NUMERIC(15,2) NOT NULL DEFAULT 0.00,

    is_blocked BOOLEAN NOT NULL DEFAULT false,
    bot_is_blocked_by_user BOOLEAN NOT NULL DEFAULT false,
    has_passed_captcha BOOLEAN NOT NULL DEFAULT false,

    registered_with_bot BIGINT NOT NULL,
    last_seen_with_bot BIGINT NOT NULL,

    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    blocked_until TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_customers_telegram_id ON customers (telegram_id);
CREATE INDEX IF NOT EXISTS idx_customers_registered_bot ON customers (registered_with_bot);
CREATE INDEX IF NOT EXISTS idx_customers_last_seen_bot ON customers (last_seen_with_bot);
CREATE INDEX IF NOT EXISTS idx_customers_last_seen_at ON customers (last_seen_at);

CREATE TRIGGER set_updated_at_customers
    BEFORE UPDATE ON customers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE OR REPLACE FUNCTION clear_expired_blocked_until()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.blocked_until IS NOT NULL AND NEW.blocked_until <= NOW() THEN
        NEW.blocked_until = NULL;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS clear_expired_blocked_until_trigger ON customers;

CREATE TRIGGER clear_expired_blocked_until_trigger
    BEFORE UPDATE ON customers
    FOR EACH ROW
    EXECUTE FUNCTION clear_expired_blocked_until();
