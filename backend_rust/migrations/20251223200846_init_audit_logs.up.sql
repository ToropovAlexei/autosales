CREATE TABLE audit_logs (
    id BIGSERIAL PRIMARY KEY,

    admin_user_id BIGINT,
    customer_id BIGINT,

    "action" TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'success' CHECK (status IN ('success', 'failed', 'denied')),

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