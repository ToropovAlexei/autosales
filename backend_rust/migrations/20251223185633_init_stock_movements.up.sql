CREATE TABLE stock_movements (
    id BIGSERIAL PRIMARY KEY,
    order_id BIGINT,
    product_id BIGINT NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('initial', 'restock', 'sale', 'return', 'adjustment')),
    quantity BIGINT NOT NULL,
    balance_after BIGINT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by BIGINT NOT NULL,

    description TEXT,
    reference_id TEXT,

    CONSTRAINT fk_stock_movements_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE RESTRICT,
    CONSTRAINT fk_stock_movements_order
        FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE SET NULL,
    CONSTRAINT fk_stock_movements_created_by
        FOREIGN KEY (created_by) REFERENCES admin_users(id) ON DELETE RESTRICT,

    CONSTRAINT chk_quantity_sign
        CHECK (
            (type IN ('initial', 'restock') AND quantity > 0) OR
            (type IN ('sale', 'return', 'adjustment') AND quantity != 0)
        ),
    CONSTRAINT chk_return_has_order
        CHECK (type != 'return' OR order_id IS NOT NULL),
    CONSTRAINT chk_sale_has_order
        CHECK (type != 'sale' OR order_id IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_stock_movements_product_id ON stock_movements (product_id);
CREATE INDEX IF NOT EXISTS idx_stock_movements_order_id ON stock_movements (order_id);
CREATE INDEX IF NOT EXISTS idx_stock_movements_created_at ON stock_movements (created_at);
CREATE INDEX IF NOT EXISTS idx_stock_movements_type ON stock_movements (type);

CREATE OR REPLACE FUNCTION update_balance_after()
RETURNS TRIGGER AS $$
DECLARE
    last_balance BIGINT;
BEGIN
    SELECT COALESCE((SELECT balance_after
                        FROM stock_movements
                        WHERE product_id = NEW.product_id
                        ORDER BY id DESC
                        LIMIT 1), 0)
    INTO last_balance;

    NEW.balance_after := last_balance + NEW.quantity;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_balance_after
    BEFORE INSERT ON stock_movements
    FOR EACH ROW
    EXECUTE FUNCTION update_balance_after();

CREATE OR REPLACE FUNCTION update_product_stock()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE products
    SET 
        stock = NEW.balance_after,
        updated_at = NOW()
    WHERE id = NEW.product_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_product_stock
    AFTER INSERT ON stock_movements
    FOR EACH ROW
    EXECUTE FUNCTION update_product_stock();