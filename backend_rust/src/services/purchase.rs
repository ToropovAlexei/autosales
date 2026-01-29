use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use rust_decimal_macros::dec;
use serde::Deserialize;
use serde_json::json;
use shared_dtos::{order::OrderStatus, product::ProductType};
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::{
        external::products::contms::{ContmsProductsProvider, ContmsProductsProviderTrait},
        repositories::{
            audit_log::AuditLogRepository, category::CategoryRepository,
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
    pub details: Option<serde_json::Value>,
    pub fulfilled_text: Option<String>,
    pub fulfilled_image_id: Option<Uuid>,
    pub price: f64,
}

#[async_trait]
pub trait PurchaseServiceTrait: Send + Sync {
    async fn purchase_product(&self, command: PurchaseProductCommand) -> ApiResult<PurchaseResult>;
}

pub struct PurchaseService<T, C, OI, O, P, CMS, US> {
    pub transactions_service: Arc<T>,
    pub customer_service: Arc<C>,
    pub order_service: Arc<O>,
    pub order_item_service: Arc<OI>,
    pub product_service: Arc<P>,
    pub contms_provider: Arc<CMS>,
    pub user_subscription_service: Arc<US>,
}

impl<T, C, OI, O, P, CMS, US> PurchaseService<T, C, OI, O, P, CMS, US>
where
    T: TransactionServiceTrait + Send + Sync,
    C: CustomerServiceTrait + Send + Sync,
    OI: OrderItemServiceTrait + Send + Sync,
    O: OrderServiceTrait + Send + Sync,
    P: ProductServiceTrait + Send + Sync,
    CMS: ContmsProductsProviderTrait + Send + Sync,
    US: UserSubscriptionServiceTrait + Send + Sync,
{
    pub fn new(
        transactions_service: Arc<T>,
        customer_service: Arc<C>,
        product_service: Arc<P>,
        order_service: Arc<O>,
        order_item_service: Arc<OI>,
        contms_provider: Arc<CMS>,
        user_subscription_service: Arc<US>,
    ) -> Self {
        Self {
            transactions_service,
            customer_service,
            product_service,
            order_item_service,
            order_service,
            contms_provider,
            user_subscription_service,
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
    >
{
    async fn purchase_product(&self, command: PurchaseProductCommand) -> ApiResult<PurchaseResult> {
        // TODO Refactor this function
        let product = self.product_service.get_by_id(command.product_id).await?;
        if product.stock < command.amount as i32 {
            return Err(ApiError::BadRequest("Not enough stock".to_string()));
        }
        let customer = self
            .customer_service
            .get_by_telegram_id(command.telegram_id)
            .await?;
        let is_subscription = product.r#type == ProductType::Subscription;
        let amount = if is_subscription { 0 } else { command.amount };
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
                details: product.details.clone(),
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
            })
            .await?;
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
            // TODO Ensure secs or millis
            let expiration_date = DateTime::from_timestamp_secs(proxy.expires).ok_or(
                ApiError::InternalServerError("Failed to parse proxy expires".to_string()),
            )?;
            let subscription = self
                    .user_subscription_service
                    .create(NewUserSubscription {
                        customer_id: customer.id,
                        details: Some(json!({ "username": proxy.name, "password": proxy.pass, "details": product.details })),
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
                details: subscription.details,
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
            details: product.details,
            fulfilled_image_id: product.fulfillment_image_id,
            fulfilled_text: product.fulfillment_text,
            product_name: product.name,
            price: total_price.to_f64().unwrap_or_default(),
        })
    }
}
