CREATE TABLE order_items (
    id BIGSERIAL PRIMARY KEY,
    order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,

    name_at_purchase TEXT NOT NULL,
    price_at_purchase NUMERIC(12,2) NOT NULL CHECK (price_at_purchase >= 0),
    quantity SMALLINT NOT NULL CHECK (quantity > 0),

    fulfillment_type TEXT NOT NULL DEFAULT 'none'
        CHECK (fulfillment_type IN ('text', 'image', 'link', 'none')),
    fulfillment_content TEXT,
    fulfillment_image_id UUID,

    details JSONB,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_order_items_order
        FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE,
    CONSTRAINT fk_order_items_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE RESTRICT,
    CONSTRAINT fk_order_items_fulfillment_image
        FOREIGN KEY (fulfillment_image_id) REFERENCES images(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items (order_id);
CREATE INDEX IF NOT EXISTS idx_order_items_product_id ON order_items (product_id);