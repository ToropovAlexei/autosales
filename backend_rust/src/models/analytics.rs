use rust_decimal::Decimal;

#[derive(Debug)]
pub struct BotAnalyticsRow {
    pub bot_id: i64,
    pub total_earnings: Decimal,
    pub purchase_count: i64,
}
