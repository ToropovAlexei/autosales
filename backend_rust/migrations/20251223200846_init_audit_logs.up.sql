CREATE TYPE audit_action AS ENUM (
    'user_login', 'user_logout', 'user_create', 'user_update', 'user_delete',
    'role_grant', 'role_revoke', 'permission_grant', 'permission_revoke',

    'product_create', 'product_update', 'product_delete', 'product_hide',
    'stock_movement_create',

    'balance_deposit', 'balance_withdrawal', 'referral_payout',
    'invoice_create', 'invoice_pay', 'invoice_expire',

    'system_settings_update', 'bot_start', 'api_call',
    'category_create', 'category_update', 'category_delete'
);

CREATE TYPE audit_status AS ENUM (
    'success',
    'failed',
    'denied'
);

CREATE TABLE audit_logs (
    id BIGSERIAL PRIMARY KEY,

    admin_user_id BIGINT,
    customer_id BIGINT,

    "action" audit_action NOT NULL,
    status audit_status NOT NULL DEFAULT 'success',

    target_table TEXT NOT NULL,
    target_id TEXT NOT NULL,

    old_values JSONB,
    new_values JSONB,

    ip_address INET,
    user_agent TEXT,
    request_id TEXT,
    error_message TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_user_exclusive
        CHECK (
            (admin_user_id IS NOT NULL AND customer_id IS NULL) OR
            (admin_user_id IS NULL AND customer_id IS NOT NULL) OR
            (admin_user_id IS NULL AND customer_id IS NULL)
        )
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_admin_user ON audit_logs (admin_user_id) WHERE admin_user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_logs_bot_user ON audit_logs (customer_id) WHERE customer_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs (action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_status ON audit_logs (status);
CREATE INDEX IF NOT EXISTS idx_audit_logs_target ON audit_logs (target_table, target_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_ip ON audit_logs USING GiST (ip_address inet_ops);