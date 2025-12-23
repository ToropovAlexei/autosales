CREATE TABLE permissions (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE CHECK (name ~ '^[a-z0-9:_-]+$'),
    "group" TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);