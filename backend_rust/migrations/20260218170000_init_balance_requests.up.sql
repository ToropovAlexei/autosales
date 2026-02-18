CREATE TABLE store_balance_requests (
    id BIGSERIAL PRIMARY KEY,
    request_type TEXT NOT NULL CHECK (request_type IN ('withdrawal', 'deposit')),
    wallet_address TEXT NOT NULL,
    amount_usdt NUMERIC(12,2) NOT NULL CHECK (amount_usdt > 0),
    fx_rate_rub_to_usdt NUMERIC(12,6) NOT NULL CHECK (fx_rate_rub_to_usdt > 0),
    amount_rub NUMERIC(12,2) NOT NULL CHECK (amount_rub > 0),
    status TEXT NOT NULL CHECK (status IN ('pending_operator', 'completed', 'rejected', 'canceled')),
    operator_tg_user_id BIGINT,
    operator_comment TEXT,
    operator_action_at TIMESTAMPTZ,
    telegram_message_id BIGINT,
    telegram_chat_id BIGINT,
    debit_transaction_id BIGINT,
    credit_transaction_id BIGINT,
    refund_transaction_id BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_store_balance_requests_debit_transaction
        FOREIGN KEY (debit_transaction_id) REFERENCES transactions(id) ON DELETE SET NULL,
    CONSTRAINT fk_store_balance_requests_credit_transaction
        FOREIGN KEY (credit_transaction_id) REFERENCES transactions(id) ON DELETE SET NULL,
    CONSTRAINT fk_store_balance_requests_refund_transaction
        FOREIGN KEY (refund_transaction_id) REFERENCES transactions(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_store_balance_requests_status ON store_balance_requests (status);
CREATE INDEX IF NOT EXISTS idx_store_balance_requests_request_type ON store_balance_requests (request_type);
CREATE INDEX IF NOT EXISTS idx_store_balance_requests_created_at ON store_balance_requests (created_at DESC);

CREATE TRIGGER trigger_update_updated_at_store_balance_requests
BEFORE UPDATE ON store_balance_requests
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

ALTER TABLE transactions
DROP CONSTRAINT IF EXISTS transactions_type_check;

ALTER TABLE transactions
ADD CONSTRAINT transactions_type_check
CHECK (type IN (
    'deposit',
    'purchase',
    'withdrawal',
    'referral_payout',
    'service_charge',
    'refund',
    'balance_request_withdrawal_debit',
    'balance_request_withdrawal_refund',
    'balance_request_deposit_credit'
));
