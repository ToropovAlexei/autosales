CREATE TABLE broadcasts (
    id BIGSERIAL PRIMARY KEY,
    status TEXT NOT NULL CHECK (status IN ('pending', 'scheduled', 'in_progress', 'completed', 'failed')),
    content_text TEXT,
    content_image_id UUID,
    filters JSONB,
    statistics JSONB,
    created_by BIGINT NOT NULL,
    scheduled_for TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_broadcasts_content_image
        FOREIGN KEY (content_image_id) REFERENCES images(id) ON DELETE SET NULL,
    CONSTRAINT fk_broadcasts_created_by
        FOREIGN KEY (created_by) REFERENCES admin_users(id) ON DELETE RESTRICT,
    CONSTRAINT chk_broadcast_has_content
        CHECK (content_text IS NOT NULL OR content_image_id IS NOT NULL)
);

CREATE INDEX idx_broadcasts_status ON broadcasts (status);
CREATE INDEX idx_broadcasts_scheduled_for ON broadcasts (scheduled_for) WHERE status = 'scheduled';
CREATE INDEX idx_broadcasts_created_by ON broadcasts (created_by);

CREATE TRIGGER set_updated_at_broadcasts
    BEFORE UPDATE ON broadcasts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();