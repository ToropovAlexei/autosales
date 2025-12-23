CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TRIGGER trigger_update_updated_at
BEFORE UPDATE ON settings
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();