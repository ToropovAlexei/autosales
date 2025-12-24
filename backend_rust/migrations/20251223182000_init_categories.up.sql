CREATE TABLE categories (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL
        CHECK (LENGTH(TRIM(name)) > 0),
    parent_id BIGINT,
    image_id UUID,

    position SMALLINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by BIGINT NOT NULL,

    CONSTRAINT fk_categories_parent
        FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE CASCADE,
    CONSTRAINT fk_categories_image
        FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE SET NULL,
    CONSTRAINT fk_categories_created_by
        FOREIGN KEY (created_by) REFERENCES admin_users(id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_categories_parent_id ON categories (parent_id);
CREATE INDEX IF NOT EXISTS idx_categories_is_active ON categories (is_active);
CREATE INDEX IF NOT EXISTS idx_categories_position ON categories (parent_id, position);

CREATE TRIGGER set_updated_at_categories
    BEFORE UPDATE ON categories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();