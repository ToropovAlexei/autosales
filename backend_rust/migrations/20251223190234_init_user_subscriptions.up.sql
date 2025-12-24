CREATE TABLE user_subscriptions (
    id BIGSERIAL PRIMARY KEY,
    customer_id BIGINT NOT NULL,
    product_id BIGINT,
    order_id BIGINT NOT NULL,

    started_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,

    cancelled_at TIMESTAMPTZ,

    next_charge_at TIMESTAMPTZ,
    renewal_order_id BIGINT,

    price_at_subscription NUMERIC(12,2) NOT NULL CHECK (price_at_subscription >= 0),
    period_days SMALLINT NOT NULL CHECK (period_days > 0),

    details JSONB,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_user_subscriptions_bot_user
        FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE RESTRICT,
    CONSTRAINT fk_user_subscriptions_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE SET NULL,
    CONSTRAINT fk_user_subscriptions_order
        FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE RESTRICT,
    CONSTRAINT fk_user_subscriptions_renewal_order
        FOREIGN KEY (renewal_order_id) REFERENCES orders(id) ON DELETE SET NULL,

    CONSTRAINT chk_expires_after_start
        CHECK (expires_at > started_at),
    CONSTRAINT chk_next_charge_after_expires
        CHECK (next_charge_at IS NULL OR next_charge_at >= expires_at)
);

CREATE INDEX IF NOT EXISTS idx_user_subscriptions_customer_id ON user_subscriptions (customer_id);
CREATE INDEX IF NOT EXISTS idx_user_subscriptions_expires_at ON user_subscriptions (expires_at);
CREATE INDEX IF NOT EXISTS idx_user_subscriptions_active ON user_subscriptions (expires_at)
    WHERE cancelled_at IS NULL AND expires_at > NOW();
CREATE INDEX IF NOT EXISTS idx_user_subscriptions_order_id ON user_subscriptions (order_id);
CREATE INDEX IF NOT EXISTS idx_user_subscriptions_product_id ON user_subscriptions (product_id);

CREATE TRIGGER set_updated_at_user_subscriptions
    BEFORE UPDATE ON user_subscriptions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();