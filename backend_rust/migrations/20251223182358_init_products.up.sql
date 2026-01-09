CREATE TABLE products (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL CHECK (LENGTH(TRIM(name)) > 0),
    base_price NUMERIC(12,2) NOT NULL CHECK (base_price >= 0),
    category_id BIGINT,
    image_id UUID,

    stock INTEGER NOT NULL DEFAULT 0 CHECK (stock >= 0),

    type TEXT NOT NULL DEFAULT 'item' CHECK (type IN ('item', 'subscription')),
    subscription_period_days SMALLINT NOT NULL DEFAULT 0
        CHECK (subscription_period_days >= 0),

    details JSONB,

    deleted_at TIMESTAMPTZ,

    fulfillment_text TEXT,
    fulfillment_image_id UUID,

    provider_name TEXT NOT NULL
        CHECK (provider_name ~ '^[a-z][a-z0-9_]{2,31}$'),
    external_id TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by BIGINT NOT NULL,

    CONSTRAINT fk_products_category
        FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL,
    CONSTRAINT fk_products_image
        FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE SET NULL,
    CONSTRAINT fk_products_fulfillment_image
        FOREIGN KEY (fulfillment_image_id) REFERENCES images(id) ON DELETE SET NULL,
    CONSTRAINT fk_products_created_by
        FOREIGN KEY (created_by) REFERENCES admin_users(id) ON DELETE RESTRICT,

    CONSTRAINT chk_subscription_period
        CHECK (
            (type = 'subscription' AND subscription_period_days > 0) OR
            (type != 'subscription' AND subscription_period_days = 0)
        )
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_products_provider_external
    ON products (provider_name, external_id)
    WHERE external_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_products_category_id ON products (category_id);
CREATE INDEX IF NOT EXISTS idx_products_name ON products (name);
CREATE INDEX IF NOT EXISTS idx_products_base_price ON products (base_price);
CREATE INDEX IF NOT EXISTS idx_products_created_at ON products (created_at DESC);

CREATE TRIGGER set_updated_at_products
    BEFORE UPDATE ON products
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();