CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE active_tokens (
    jti UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id BIGINT NOT NULL,
    token_type TEXT NOT NULL CHECK (token_type IN ('access', 'refresh')),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ,

    CONSTRAINT fk_active_tokens_user
        FOREIGN KEY (user_id) REFERENCES admin_users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_active_tokens_user_id ON active_tokens (user_id);
CREATE INDEX IF NOT EXISTS idx_active_tokens_expires_at ON active_tokens (expires_at) WHERE revoked_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_active_tokens_revoked ON active_tokens (revoked_at) WHERE revoked_at IS NOT NULL;