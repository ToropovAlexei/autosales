CREATE TABLE roles (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE CHECK (name ~ '^[a-z0-9_-]+$'),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS roles_name_idx ON roles (name);

CREATE TRIGGER trigger_update_updated_at
BEFORE UPDATE ON roles
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();