CREATE TYPE order_status AS ENUM (
    'created',
    'paid',
    'fulfilled',
    'cancelled',
    'refunded'
);

CREATE TABLE orders (
    id BIGSERIAL PRIMARY KEY,
    customer_id BIGINT NOT NULL,
    amount NUMERIC(12,2) NOT NULL CHECK (amount >= 0),
    currency CHAR(3) NOT NULL DEFAULT 'RUB',
    status order_status NOT NULL DEFAULT 'created',
    bot_id BIGINT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    paid_at TIMESTAMPTZ,
    fulfilled_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,

    CONSTRAINT fk_orders_user
        FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE RESTRICT,
    CONSTRAINT fk_orders_bot
        FOREIGN KEY (bot_id) REFERENCES bots(id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_orders_customer_id ON orders (customer_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders (status);
CREATE INDEX IF NOT EXISTS idx_orders_bot_id ON orders (bot_id);
CREATE INDEX IF NOT EXISTS idx_orders_created_at ON orders (created_at DESC);

CREATE TRIGGER set_updated_at_orders
    BEFORE UPDATE ON orders
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();