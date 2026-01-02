CREATE TABLE images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    original_filename TEXT,
    hash TEXT NOT NULL UNIQUE,
    mime_type TEXT NOT NULL
        CHECK (mime_type ~ '^image/(jpeg|png|webp|gif|svg\+xml)$'),
    file_size BIGINT NOT NULL
        CHECK (file_size > 0 AND file_size <= 20971520),
    width SMALLINT,
    height SMALLINT,
    context TEXT NOT NULL DEFAULT 'product'
        CHECK (context IN ('product', 'category', 'fulfillment', 'other')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by BIGINT NOT NULL,
    deleted_at TIMESTAMPTZ,

    CONSTRAINT fk_images_created_by
        FOREIGN KEY (created_by) REFERENCES admin_users(id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_images_deleted_at ON images (deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_images_context ON images (context);
CREATE UNIQUE INDEX IF NOT EXISTS idx_images_hash ON images (hash);
CREATE INDEX IF NOT EXISTS idx_images_created_at ON images (created_at);