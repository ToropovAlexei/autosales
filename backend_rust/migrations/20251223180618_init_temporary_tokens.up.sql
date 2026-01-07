CREATE TABLE temporary_tokens (
    token UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id BIGINT NOT NULL,
    purpose TEXT NOT NULL CHECK (purpose IN ('2fa', 'password_reset')),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    used_at TIMESTAMPTZ,

    CONSTRAINT fk_temporary_tokens_user
        FOREIGN KEY (user_id) REFERENCES admin_users(id) ON DELETE CASCADE,
    CONSTRAINT chk_temporary_not_used_if_expired
        CHECK (used_at IS NULL OR used_at <= expires_at)
);

CREATE INDEX IF NOT EXISTS idx_temporary_tokens_user_id ON temporary_tokens (user_id);
CREATE INDEX IF NOT EXISTS idx_temporary_tokens_expires_at ON temporary_tokens (expires_at) WHERE used_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_temporary_tokens_used ON temporary_tokens (used_at) WHERE used_at IS NOT NULL;