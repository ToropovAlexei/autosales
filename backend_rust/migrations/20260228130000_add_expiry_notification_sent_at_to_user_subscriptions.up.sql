ALTER TABLE user_subscriptions
    ADD COLUMN expiry_notification_sent_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_user_subscriptions_expiry_notification_sent_at
    ON user_subscriptions (expiry_notification_sent_at);
