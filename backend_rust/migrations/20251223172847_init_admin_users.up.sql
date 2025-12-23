CREATE TABLE admin_users (
    id BIGSERIAL PRIMARY KEY,
    login TEXT NOT NULL UNIQUE
        CHECK (login ~ '^[a-z][a-z0-9._-]{2,31}$'),
    hashed_password TEXT NOT NULL,
    two_fa_secret TEXT NOT NULL,
    telegram_id BIGINT UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_admin_users_login ON admin_users (login);
CREATE INDEX IF NOT EXISTS idx_admin_users_telegram_id ON admin_users (telegram_id);
CREATE INDEX IF NOT EXISTS idx_admin_users_deleted_at ON admin_users (deleted_at) WHERE deleted_at IS NOT NULL;

CREATE TRIGGER set_updated_at_admin_users
    BEFORE UPDATE ON admin_users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();