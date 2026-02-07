CREATE OR REPLACE FUNCTION clear_expired_blocked_until()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.blocked_until IS NOT NULL AND NEW.blocked_until <= NOW() THEN
        NEW.blocked_until = NULL;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS clear_expired_blocked_until_trigger ON customers;

CREATE TRIGGER clear_expired_blocked_until_trigger
    BEFORE UPDATE ON customers
    FOR EACH ROW
    EXECUTE FUNCTION clear_expired_blocked_until();
