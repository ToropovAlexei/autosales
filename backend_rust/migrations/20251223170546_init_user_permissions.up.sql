CREATE TYPE permission_effect AS ENUM ('allow', 'deny');

CREATE TABLE user_permissions (
    user_id BIGINT NOT NULL,
    permission_id BIGINT NOT NULL,
    effect permission_effect NOT NULL DEFAULT 'allow',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, permission_id)
);

ALTER TABLE user_permissions
    ADD CONSTRAINT fk_user_permissions_user
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    ADD CONSTRAINT fk_user_permissions_permission
        FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE;

CREATE TRIGGER trigger_update_updated_at
BEFORE UPDATE ON user_permissions
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();