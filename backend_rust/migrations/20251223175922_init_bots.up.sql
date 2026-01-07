CREATE TABLE bots (
    id BIGSERIAL PRIMARY KEY,
    owner_id BIGINT,
    token TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE
        CHECK (username ~ '^[A-Za-z][A-Za-z0-9_]{4,31}$'),
    type TEXT NOT NULL CHECK (type IN ('main', 'referral')),
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    referral_percentage NUMERIC(5,2) NOT NULL DEFAULT 0.00
        CHECK (referral_percentage BETWEEN 0.00 AND 100.00),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by BIGINT
);

ALTER TABLE bots
    ADD CONSTRAINT fk_bots_created_by
        FOREIGN KEY (created_by) REFERENCES admin_users(id) ON DELETE RESTRICT;

CREATE INDEX IF NOT EXISTS idx_bots_owner_id ON bots (owner_id);
CREATE INDEX IF NOT EXISTS idx_bots_type ON bots (type);
CREATE INDEX IF NOT EXISTS idx_bots_is_active ON bots (is_active);
CREATE INDEX IF NOT EXISTS idx_bots_is_primary ON bots (is_primary);