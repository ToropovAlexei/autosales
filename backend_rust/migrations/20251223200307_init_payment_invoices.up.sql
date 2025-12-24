CREATE TYPE invoice_status AS ENUM (
    'pending',
    'completed',
    'failed',
    'expired',
    'refunded'
);

CREATE TABLE payment_invoices (
    id BIGSERIAL PRIMARY KEY,
    customer_id BIGINT NOT NULL,

    original_amount NUMERIC(12,2) NOT NULL CHECK (original_amount > 0),
    amount NUMERIC(12,2) NOT NULL CHECK (amount >= original_amount),

    status invoice_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,

    gateway TEXT NOT NULL
        CHECK (gateway ~ '^[a-z][a-z0-9_]{2,31}$'),
    gateway_invoice_id TEXT NOT NULL,
    order_id UUID NOT NULL DEFAULT gen_random_uuid(),

    payment_details JSONB,

    bot_message_id BIGINT,
    notification_sent_at TIMESTAMPTZ,

    CONSTRAINT fk_payment_invoices_bot_user
        FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_payment_invoices_customer_id ON payment_invoices (customer_id);
CREATE INDEX IF NOT EXISTS idx_payment_invoices_deleted_at ON payment_invoices (deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_payment_invoices_gateway ON payment_invoices (gateway);
CREATE UNIQUE INDEX IF NOT EXISTS idx_payment_invoices_gateway_invoice_id ON payment_invoices (gateway, gateway_invoice_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_payment_invoices_order_id ON payment_invoices (order_id);
CREATE INDEX IF NOT EXISTS idx_payment_invoices_status ON payment_invoices (status);
CREATE INDEX IF NOT EXISTS idx_payment_invoices_expires_at ON payment_invoices (expires_at);

CREATE TRIGGER set_updated_at_payment_invoices
    BEFORE UPDATE ON payment_invoices
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();