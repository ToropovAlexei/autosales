use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use rust_decimal_macros::dec;
use serde::Deserialize;
use shared_dtos::{
    order::{OrderStatus, PurchaseDetails},
    product::{ProductDetails, ProductType},
    user_subscription::UserSubscriptionDetails,
};
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::{
        external::products::contms::{ContmsProductsProvider, ContmsProductsProviderTrait},
        repositories::{
            audit_log::AuditLogRepository, bot::BotRepository, category::CategoryRepository,
            customer::CustomerRepository, order::OrderRepository, order_item::OrderItemRepository,
            products::ProductRepository, settings::SettingsRepository,
            stock_movement::StockMovementRepository, transaction::TransactionRepository,
            user_subscription::UserSubscriptionRepository,
        },
    },
    models::{
        order::NewOrder,
        order_item::NewOrderItem,
        transaction::{NewTransaction, TransactionType},
        user_subscription::NewUserSubscription,
    },
    services::{
        audit_log::AuditLogService,
        bot::{BotService, BotServiceTrait},
        category::CategoryService,
        customer::{CustomerService, CustomerServiceTrait},
        order::{OrderService, OrderServiceTrait},
        order_item::{OrderItemService, OrderItemServiceTrait},
        product::{ProductService, ProductServiceTrait},
        transaction::{TransactionService, TransactionServiceTrait},
        user_subscription::{UserSubscriptionService, UserSubscriptionServiceTrait},
    },
};

type AuditLogServiceShort = AuditLogService<AuditLogRepository>;

type ProductServiceShort = ProductService<
    ProductRepository,
    StockMovementRepository,
    AuditLogServiceShort,
    SettingsRepository,
    CategoryService<CategoryRepository, AuditLogServiceShort>,
>;

#[derive(Debug, Deserialize)]
pub struct PurchaseProductCommand {
    pub product_id: i64,
    pub amount: i64,
    pub telegram_id: i64,
    pub bot_id: i64,
}

#[derive(Debug, Clone)]
pub struct PurchaseResult {
    pub product_name: String,
    pub balance: f64,
    pub details: Option<PurchaseDetails>,
    pub fulfilled_text: Option<String>,
    pub fulfilled_image_id: Option<Uuid>,
    pub price: f64,
}

#[async_trait]
pub trait PurchaseServiceTrait: Send + Sync {
    async fn purchase_product(&self, command: PurchaseProductCommand) -> ApiResult<PurchaseResult>;
}

pub struct PurchaseService<T, C, OI, O, P, CMS, US, B> {
    pub transactions_service: Arc<T>,
    pub customer_service: Arc<C>,
    pub order_service: Arc<O>,
    pub order_item_service: Arc<OI>,
    pub product_service: Arc<P>,
    pub contms_provider: Arc<CMS>,
    pub user_subscription_service: Arc<US>,
    pub bot_service: Arc<B>,
}

impl<T, C, OI, O, P, CMS, US, B> PurchaseService<T, C, OI, O, P, CMS, US, B>
where
    T: TransactionServiceTrait + Send + Sync,
    C: CustomerServiceTrait + Send + Sync,
    OI: OrderItemServiceTrait + Send + Sync,
    O: OrderServiceTrait + Send + Sync,
    P: ProductServiceTrait + Send + Sync,
    CMS: ContmsProductsProviderTrait + Send + Sync,
    US: UserSubscriptionServiceTrait + Send + Sync,
    B: BotServiceTrait + Send + Sync,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        transactions_service: Arc<T>,
        customer_service: Arc<C>,
        product_service: Arc<P>,
        order_service: Arc<O>,
        order_item_service: Arc<OI>,
        contms_provider: Arc<CMS>,
        user_subscription_service: Arc<US>,
        bot_service: Arc<B>,
    ) -> Self {
        Self {
            transactions_service,
            customer_service,
            product_service,
            order_item_service,
            order_service,
            contms_provider,
            user_subscription_service,
            bot_service,
        }
    }
}

#[async_trait]
impl PurchaseServiceTrait
    for PurchaseService<
        TransactionService<TransactionRepository>,
        CustomerService<CustomerRepository, AuditLogServiceShort>,
        OrderItemService<OrderItemRepository, StockMovementRepository>,
        OrderService<OrderRepository, OrderItemRepository>,
        ProductServiceShort,
        ContmsProductsProvider,
        UserSubscriptionService<UserSubscriptionRepository>,
        BotService<
            BotRepository,
            SettingsRepository,
            AuditLogService<AuditLogRepository>,
            TransactionRepository,
        >,
    >
{
    async fn purchase_product(&self, command: PurchaseProductCommand) -> ApiResult<PurchaseResult> {
        let bot = self.bot_service.get_by_id(command.bot_id).await?;
        // TODO Refactor this function
        let product = self.product_service.get_by_id(command.product_id).await?;
        // We should check if there is enough stock only for internal products
        if product.stock < command.amount as i32
            && product.r#type != ProductType::Subscription
            && product.external_id.is_none()
        {
            return Err(ApiError::BadRequest("Not enough stock".to_string()));
        }
        let customer = self
            .customer_service
            .get_by_telegram_id(command.telegram_id)
            .await?;
        let is_subscription = product.r#type == ProductType::Subscription;
        let amount = if is_subscription { 1 } else { command.amount };
        let total_price = product.price * Decimal::from(amount);
        if customer.balance < total_price {
            return Err(ApiError::BadRequest("Not enough balance".to_string()));
        }
        let order = self
            .order_service
            .create(NewOrder {
                amount: total_price,
                bot_id: command.bot_id,
                currency: "RUB".to_string(),
                customer_id: customer.id,
                paid_at: Some(Utc::now()),      // As it buy from balance
                fulfilled_at: Some(Utc::now()), // We send fulfillment immediately
                status: OrderStatus::Fulfilled,
            })
            .await?;
        self.order_item_service
            .create(NewOrderItem {
                order_id: order.id,
                product_id: product.id,
                details: product
                    .details
                    .clone()
                    .map(serde_json::to_value)
                    .transpose()
                    .map_err(|e| {
                        ApiError::InternalServerError(format!("Failed to serialize details: {}", e))
                    })?,
                name_at_purchase: product.name.clone(),
                price_at_purchase: product.price,
                fulfillment_content: product.fulfillment_text.clone(),
                fulfillment_image_id: product.fulfillment_image_id,
                fulfillment_type: "none".to_string(), // TODO remove this redundancy
                quantity: amount as i16,
            })
            .await?;
        let transaction = self
            .transactions_service
            .create(NewTransaction {
                amount: -total_price,
                customer_id: Some(customer.id),
                order_id: Some(order.id),
                r#type: TransactionType::Purchase,
                store_balance_delta: dec!(0), // There is no comission for purchase
                platform_commission: dec!(0), // There is no comission for purchase
                gateway_commission: dec!(0),  // There is no comission for purchase
                description: None,
                payment_gateway: None,
                details: None,
                bot_id: None,
            })
            .await?;

        // If customer buy from self bot, there is no need to create referral payout
        if let Some(owner_id) = bot.owner_id
            && customer.id != owner_id
        {
            // Add balance to referral owner
            self.transactions_service
                .create(NewTransaction {
                    amount: total_price * bot.referral_percentage / dec!(100),
                    customer_id: Some(owner_id),
                    order_id: Some(order.id),
                    r#type: TransactionType::ReferralPayout,
                    store_balance_delta: dec!(0),
                    platform_commission: dec!(0),
                    gateway_commission: dec!(0),
                    description: None,
                    payment_gateway: None,
                    details: None,
                    bot_id: Some(command.bot_id),
                })
                .await?;
        }

        if is_subscription
            && product.provider_name == "contms"
            && let Some(external_id) = product.external_id
        {
            let proxy = self
                .contms_provider
                .subscribe_to_proxy(
                    &external_id,
                    Duration::days(product.subscription_period_days as i64),
                )
                .await
                .map_err(ApiError::InternalServerError)?;
            let expiration_date = DateTime::from_timestamp_millis(proxy.expires).ok_or(
                ApiError::InternalServerError("Failed to parse proxy expires".to_string()),
            )?;
            let (host, port) = match product.details {
                Some(value) => match value {
                    ProductDetails::ContMs { host, port } => (host, port),
                },
                None => {
                    return Err(ApiError::BadRequest("Invalid product details".to_string()));
                }
            };
            let subscription_details = UserSubscriptionDetails::ContMs {
                host,
                port,
                username: proxy.name,
                password: proxy.pass,
            };
            self.user_subscription_service
                .create(NewUserSubscription {
                    customer_id: customer.id,
                    details: Some(serde_json::to_value(subscription_details.clone()).map_err(
                        |e| ApiError::BadRequest(format!("Failed to serialize details: {}", e)),
                    )?),
                    expires_at: expiration_date,
                    next_charge_at: Some(expiration_date),
                    order_id: order.id,
                    period_days: product.subscription_period_days,
                    price_at_subscription: product.price,
                    product_id: Some(product.id),
                    started_at: Utc::now(),
                })
                .await?;

            return Ok(PurchaseResult {
                balance: transaction
                    .user_balance_after
                    .unwrap_or_default()
                    .to_f64()
                    .unwrap_or_default(),
                details: Some(PurchaseDetails::UserSubscriptionDetails(
                    subscription_details,
                )),
                fulfilled_image_id: product.fulfillment_image_id,
                fulfilled_text: product.fulfillment_text,
                product_name: product.name,
                price: total_price.to_f64().unwrap_or_default(),
            });
        };

        Ok(PurchaseResult {
            balance: transaction
                .user_balance_after
                .unwrap_or_default()
                .to_f64()
                .unwrap_or_default(),
            details: product.details.map(PurchaseDetails::ProductDetails),
            fulfilled_image_id: product.fulfillment_image_id,
            fulfilled_text: product.fulfillment_text,
            product_name: product.name,
            price: total_price.to_f64().unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        errors::api::ApiError,
        infrastructure::repositories::{
            audit_log::AuditLogRepository,
            bot::BotRepository,
            category::CategoryRepository,
            customer::CustomerRepository,
            order::OrderRepository,
            order_item::OrderItemRepository,
            products::ProductRepository,
            settings::{SettingsRepository, SettingsRepositoryTrait},
            stock_movement::StockMovementRepository,
            transaction::TransactionRepository,
            user_subscription::UserSubscriptionRepository,
        },
        models::{customer::CustomerRow, settings::UpdateSettings},
        services::{
            bot::BotService, category::CategoryService, customer::CustomerService,
            order::OrderService, order_item::OrderItemService, product::ProductService,
            transaction::TransactionService, user_subscription::UserSubscriptionService,
        },
    };
    use chrono::Utc;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use sqlx::PgPool;
    use std::str::FromStr;
    use std::sync::Arc;

    type ProductServiceShort = ProductService<
        ProductRepository,
        StockMovementRepository,
        AuditLogService<AuditLogRepository>,
        SettingsRepository,
        CategoryService<CategoryRepository, AuditLogService<AuditLogRepository>>,
    >;

    type BotServiceShort = BotService<
        BotRepository,
        SettingsRepository,
        AuditLogService<AuditLogRepository>,
        TransactionRepository,
    >;

    type PurchaseServiceShort = PurchaseService<
        TransactionService<TransactionRepository>,
        CustomerService<CustomerRepository, AuditLogServiceShort>,
        OrderItemService<OrderItemRepository, StockMovementRepository>,
        OrderService<OrderRepository, OrderItemRepository>,
        ProductServiceShort,
        ContmsProductsProvider,
        UserSubscriptionService<UserSubscriptionRepository>,
        BotServiceShort,
    >;

    #[derive(Debug)]
    struct ReferralTx {
        amount: Decimal,
        customer_id: Option<i64>,
        bot_id: Option<i64>,
    }

    async fn create_customer(pool: &PgPool, telegram_id: i64, balance: &str) -> CustomerRow {
        sqlx::query_as!(
            CustomerRow,
            r#"
            INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot, balance)
            VALUES ($1, 1, 1, $2)
            RETURNING *
            "#,
            telegram_id,
            Decimal::from_str(balance).unwrap()
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn credit_customer(pool: &PgPool, customer_id: i64, amount: &str) {
        sqlx::query!(
            r#"
            INSERT INTO transactions (
                customer_id, order_id, type, amount, store_balance_delta,
                platform_commission, gateway_commission, bot_id
            )
            VALUES ($1, NULL, 'deposit', $2, $2, 0, 0, NULL)
            "#,
            customer_id,
            Decimal::from_str(amount).unwrap()
        )
        .execute(pool)
        .await
        .unwrap();
    }

    async fn create_bot(
        pool: &PgPool,
        owner_id: Option<i64>,
        token: &str,
        username: &str,
        referral_percentage: &str,
    ) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO bots (
                owner_id, token, username, type, is_active, is_primary, referral_percentage, created_by
            )
            VALUES ($1, $2, $3, 'referral', true, false, $4, 1)
            RETURNING id
            "#,
            owner_id,
            token,
            username,
            Decimal::from_str(referral_percentage).unwrap()
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_product(pool: &PgPool, name: &str, base_price: &str, stock: i32) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO products (
                name, base_price, category_id, image_id, stock, type,
                subscription_period_days, details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by, created_at, updated_at
            )
            VALUES ($1, $2, NULL, NULL, $3, 'item', 0, NULL, NULL, NULL, 'internal', NULL, 1, $4, $4)
            RETURNING id
            "#,
            name,
            Decimal::from_str(base_price).unwrap(),
            stock,
            Utc::now()
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn set_pricing_settings(pool: &PgPool, global_markup: &str, gateway_markup: &str) {
        let settings_repo = SettingsRepository::new(Arc::new(pool.clone()));
        settings_repo
            .update(UpdateSettings {
                pricing_global_markup: Some(Decimal::from_str(global_markup).unwrap()),
                pricing_gateway_markup: Some(Decimal::from_str(gateway_markup).unwrap()),
                ..Default::default()
            })
            .await
            .unwrap();
    }

    fn build_service(pool: &PgPool) -> PurchaseServiceShort {
        let pool = Arc::new(pool.clone());

        let audit_log_service = Arc::new(AuditLogService::new(Arc::new(AuditLogRepository::new(
            pool.clone(),
        ))));
        let settings_repo = Arc::new(SettingsRepository::new(pool.clone()));
        let category_service = Arc::new(CategoryService::new(
            Arc::new(CategoryRepository::new(pool.clone())),
            audit_log_service.clone(),
        ));
        let product_service = Arc::new(ProductService::new(
            Arc::new(ProductRepository::new(pool.clone())),
            Arc::new(StockMovementRepository::new(pool.clone())),
            settings_repo.clone(),
            audit_log_service.clone(),
            category_service,
        ));
        let order_item_service = Arc::new(OrderItemService::new(
            Arc::new(OrderItemRepository::new(pool.clone())),
            Arc::new(StockMovementRepository::new(pool.clone())),
        ));
        let order_service = Arc::new(OrderService::new(
            Arc::new(OrderRepository::new(pool.clone())),
            Arc::new(OrderItemRepository::new(pool.clone())),
        ));
        let transaction_service = Arc::new(TransactionService::new(Arc::new(
            TransactionRepository::new(pool.clone()),
        )));
        let customer_service = Arc::new(CustomerService::new(
            Arc::new(CustomerRepository::new(pool.clone())),
            audit_log_service.clone(),
        ));
        let user_subscription_service = Arc::new(UserSubscriptionService::new(Arc::new(
            UserSubscriptionRepository::new(pool.clone()),
        )));
        let bot_service = Arc::new(BotService::new(
            Arc::new(BotRepository::new(pool.clone())),
            settings_repo.clone(),
            Arc::new(TransactionRepository::new(pool.clone())),
            audit_log_service,
            Arc::new(reqwest::Client::new()),
        ));
        let contms_provider = Arc::new(ContmsProductsProvider::new(
            Arc::new(reqwest::Client::new()),
            "http://localhost".to_string(),
        ));

        PurchaseService::new(
            transaction_service,
            customer_service,
            product_service,
            order_service,
            order_item_service,
            contms_provider,
            user_subscription_service,
            bot_service,
        )
    }

    #[sqlx::test]
    async fn test_purchase_main_bot_no_referral(pool: PgPool) {
        let service = build_service(&pool);
        let buyer = create_customer(&pool, 101, "0.00").await;
        credit_customer(&pool, buyer.id, "500.00").await;
        let bot_id = create_bot(&pool, None, "main_bot_token", "main_bot", "0").await;
        let product_id = create_product(&pool, "Test product", "100.00", 10).await;

        let result = service
            .purchase_product(PurchaseProductCommand {
                product_id,
                amount: 1,
                telegram_id: buyer.telegram_id,
                bot_id,
            })
            .await
            .unwrap();

        assert_eq!(result.price, 100.0);

        let order_amount = sqlx::query_scalar!(
            "SELECT amount FROM orders WHERE customer_id = $1 AND bot_id = $2",
            buyer.id,
            bot_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(order_amount, Decimal::from_str("100.00").unwrap());

        let tx_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM transactions WHERE order_id IN (SELECT id FROM orders WHERE customer_id = $1 AND bot_id = $2)",
            buyer.id,
            bot_id
        )
        .fetch_one(&pool)
        .await
        .unwrap().unwrap();
        assert_eq!(tx_count, 1);

        let updated_balance =
            sqlx::query_scalar!("SELECT balance FROM customers WHERE id = $1", buyer.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(updated_balance, Decimal::from_str("400.00").unwrap());
    }

    #[sqlx::test]
    async fn test_purchase_referral_bot_payout(pool: PgPool) {
        let service = build_service(&pool);
        let owner = create_customer(&pool, 201, "0.00").await;
        let buyer = create_customer(&pool, 202, "0.00").await;
        credit_customer(&pool, buyer.id, "500.00").await;
        let bot_id = create_bot(&pool, Some(owner.id), "ref_bot_token", "ref_bot", "10.0").await;
        let product_id = create_product(&pool, "Referral product", "100.00", 10).await;

        service
            .purchase_product(PurchaseProductCommand {
                product_id,
                amount: 1,
                telegram_id: buyer.telegram_id,
                bot_id,
            })
            .await
            .unwrap();

        let referral_tx = sqlx::query_as!(
            ReferralTx,
            "SELECT amount, customer_id, bot_id FROM transactions WHERE type = 'referral_payout' AND bot_id = $1",
            bot_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(referral_tx.customer_id, Some(owner.id));
        assert_eq!(referral_tx.bot_id, Some(bot_id));
        assert_eq!(referral_tx.amount, Decimal::from_str("10.00").unwrap());

        let owner_balance =
            sqlx::query_scalar!("SELECT balance FROM customers WHERE id = $1", owner.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(owner_balance, Decimal::from_str("10.00").unwrap());
    }

    #[sqlx::test]
    async fn test_purchase_applies_markups(pool: PgPool) {
        set_pricing_settings(&pool, "20.0", "20.0").await;
        let service = build_service(&pool);
        let buyer = create_customer(&pool, 301, "0.00").await;
        credit_customer(&pool, buyer.id, "500.00").await;
        let bot_id = create_bot(&pool, None, "markup_bot_token", "markup_bot", "0").await;
        let product_id = create_product(&pool, "Markup product", "100.00", 10).await;

        service
            .purchase_product(PurchaseProductCommand {
                product_id,
                amount: 1,
                telegram_id: buyer.telegram_id,
                bot_id,
            })
            .await
            .unwrap();

        let expected = dec!(150.00);

        let order_amount = sqlx::query_scalar!(
            "SELECT amount FROM orders WHERE customer_id = $1 AND bot_id = $2",
            buyer.id,
            bot_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(order_amount, expected);

        let updated_balance =
            sqlx::query_scalar!("SELECT balance FROM customers WHERE id = $1", buyer.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(updated_balance, Decimal::from_str("350.00").unwrap());
    }

    #[sqlx::test]
    async fn test_purchase_insufficient_balance(pool: PgPool) {
        let service = build_service(&pool);
        let buyer = create_customer(&pool, 401, "0.00").await;
        credit_customer(&pool, buyer.id, "50.00").await;
        let bot_id = create_bot(&pool, None, "low_balance_bot", "low_balance_bot", "0").await;
        let product_id = create_product(&pool, "Expensive product", "100.00", 10).await;

        let err = service
            .purchase_product(PurchaseProductCommand {
                product_id,
                amount: 1,
                telegram_id: buyer.telegram_id,
                bot_id,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, ApiError::BadRequest(_)));
    }
}
