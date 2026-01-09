use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use rust_decimal_macros::dec;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::{
        audit_log::AuditLogRepository,
        category::CategoryRepository,
        products::{ProductRepository, ProductRepositoryTrait},
        settings::{SettingsRepository, SettingsRepositoryTrait},
        stock_movement::{StockMovementRepository, StockMovementRepositoryTrait},
    },
    middlewares::context::RequestContext,
    models::{
        audit_log::{AuditAction, AuditStatus, NewAuditLog},
        common::PaginatedResult,
        product::{NewProduct, ProductListQuery, ProductRow, ProductType, UpdateProduct},
        settings::Settings,
        stock_movement::{NewStockMovement, StockMovementType},
    },
    presentation::admin::dtos::product::UploadedProductCSV,
    services::{
        audit_log::{AuditLogService, AuditLogServiceTrait},
        category::{CategoryService, CategoryServiceTrait, CreateCategorySequenceCommand},
    },
};

#[derive(Debug, Clone, Serialize)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub price: Decimal,
    pub base_price: Decimal,
    pub category_id: Option<i64>,
    pub stock: i32,
    pub image_id: Option<Uuid>,
    pub r#type: ProductType,
    pub subscription_period_days: i16,
    pub details: Option<serde_json::Value>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub fulfillment_text: Option<String>,
    pub fulfillment_image_id: Option<Uuid>,
    pub provider_name: String,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct CreateProductCommand {
    pub name: String,
    pub base_price: Decimal,
    pub category_id: i64,
    pub image_id: Option<Uuid>,
    pub r#type: ProductType,
    pub subscription_period_days: Option<i16>,
    pub details: Option<serde_json::Value>,
    pub fulfillment_text: Option<String>,
    pub fulfillment_image_id: Option<Uuid>,
    pub provider_name: String,
    pub external_id: Option<String>,
    pub created_by: i64,
    pub initial_stock: Option<i64>,
    pub ctx: Option<RequestContext>,
}

#[derive(Debug)]
pub struct UpdateProductCommand {
    pub id: i64,
    pub name: Option<String>,
    pub base_price: Option<Decimal>,
    pub category_id: Option<i64>,
    pub image_id: Option<Option<Uuid>>,
    pub r#type: Option<ProductType>,
    pub subscription_period_days: Option<i16>,
    pub details: Option<Option<serde_json::Value>>,
    pub fulfillment_text: Option<Option<String>>,
    pub fulfillment_image_id: Option<Option<Uuid>>,
    pub external_id: Option<Option<String>>,
    pub stock: Option<i64>,
    pub updated_by: i64,
}

#[derive(Debug)]
pub struct DeleteProductCommand {
    pub id: i64,
    pub deleted_by: i64,
    pub ctx: Option<RequestContext>,
}

#[derive(Debug)]
pub struct UploadProductsCommand {
    pub products: Vec<UploadedProductCSV>,
    pub created_by: i64,
    pub ctx: Option<RequestContext>,
}

#[async_trait]
pub trait ProductServiceTrait: Send + Sync {
    async fn get_list(&self, query: ProductListQuery) -> ApiResult<PaginatedResult<Product>>;
    async fn create(&self, command: CreateProductCommand) -> ApiResult<Product>;
    async fn get_by_id(&self, id: i64) -> ApiResult<Product>;
    async fn update(
        &self,
        command: UpdateProductCommand,
        ctx: RequestContext,
    ) -> ApiResult<Product>;
    async fn delete(&self, command: DeleteProductCommand) -> ApiResult<()>;
    async fn get_for_external_provider(
        &self,
        provider_name: &str,
        external_id: &str,
    ) -> ApiResult<Product>;
    async fn get_all_external_provider(&self, provider_name: &str) -> ApiResult<Vec<Product>>;
    async fn upload_products(
        &self,
        command: UploadProductsCommand,
    ) -> ApiResult<(Vec<Product>, Vec<String>)>;
}

pub struct ProductService<R, S, A, T, C> {
    product_repo: Arc<R>,
    stock_movement_repo: Arc<S>,
    settings_repo: Arc<T>,
    category_service: Arc<C>,
    audit_log_service: Arc<A>,
}

impl<R, S, A, T, C> ProductService<R, S, A, T, C>
where
    R: ProductRepositoryTrait + Send + Sync,
    S: StockMovementRepositoryTrait + Send + Sync,
    A: AuditLogServiceTrait + Send + Sync,
    C: CategoryServiceTrait + Send + Sync,
{
    pub fn new(
        product_repo: Arc<R>,
        stock_movement_repo: Arc<S>,
        settings_repo: Arc<T>,
        audit_log_service: Arc<A>,
        category_service: Arc<C>,
    ) -> Self {
        Self {
            product_repo,
            stock_movement_repo,
            settings_repo,
            audit_log_service,
            category_service,
        }
    }
}

#[async_trait]
impl ProductServiceTrait
    for ProductService<
        ProductRepository,
        StockMovementRepository,
        AuditLogService<AuditLogRepository>,
        SettingsRepository,
        CategoryService<CategoryRepository, AuditLogService<AuditLogRepository>>,
    >
{
    async fn get_list(&self, query: ProductListQuery) -> ApiResult<PaginatedResult<Product>> {
        let settings = self.settings_repo.load_settings().await?;
        self.product_repo
            .get_list(query)
            .await
            .map_err(ApiError::from)
            .map(|res| PaginatedResult {
                items: res
                    .items
                    .iter()
                    .map(|row| from_product_row(row.clone(), &settings.clone()))
                    .collect(),
                total: res.total,
            })
    }

    async fn create(&self, command: CreateProductCommand) -> ApiResult<Product> {
        let settings = self.settings_repo.load_settings().await?;
        let created = self
            .product_repo
            .create(NewProduct {
                category_id: command.category_id,
                created_by: command.created_by,
                details: command.details,
                external_id: command.external_id,
                fulfillment_image_id: command.fulfillment_image_id,
                fulfillment_text: command.fulfillment_text,
                image_id: command.image_id,
                name: command.name,
                base_price: command.base_price,
                provider_name: command.provider_name,
                subscription_period_days: command.subscription_period_days.unwrap_or_default(),
                r#type: command.r#type,
            })
            .await?;

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::ProductCreate,
                status: AuditStatus::Success,
                admin_user_id: Some(command.created_by),
                customer_id: None,
                error_message: None,
                new_values: serde_json::to_value(created.clone()).ok(),
                old_values: None,
                target_id: created.id.to_string(),
                target_table: "products".to_string(),
                ip_address: command.ctx.clone().and_then(|ctx| ctx.ip_address),
                request_id: command.ctx.clone().map(|ctx| ctx.request_id),
                user_agent: command.ctx.clone().and_then(|ctx| ctx.user_agent),
            })
            .await?;

        if let Some(initial_stock) = command.initial_stock {
            let stock_movement = self
                .stock_movement_repo
                .create(NewStockMovement {
                    product_id: created.id,
                    quantity: initial_stock,
                    created_by: command.created_by,
                    r#type: StockMovementType::Initial,
                    order_id: None,
                    description: None,
                    reference_id: None,
                })
                .await?;

            self.audit_log_service
                .create(NewAuditLog {
                    action: AuditAction::StockMovementCreate,
                    status: AuditStatus::Success,
                    admin_user_id: Some(command.created_by),
                    customer_id: None,
                    error_message: None,
                    new_values: serde_json::to_value(stock_movement.clone()).ok(),
                    old_values: None,
                    target_id: created.id.to_string(),
                    target_table: "stock_movements".to_string(),
                    ip_address: command.ctx.clone().and_then(|ctx| ctx.ip_address),
                    request_id: command.ctx.clone().map(|ctx| ctx.request_id),
                    user_agent: command.ctx.and_then(|ctx| ctx.user_agent),
                })
                .await?;
        }

        Ok(from_product_row(created, &settings))
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<Product> {
        let settings = self.settings_repo.load_settings().await?;
        let res = self.product_repo.get_by_id(id).await?;
        Ok(from_product_row(res, &settings))
    }

    async fn update(
        &self,
        command: UpdateProductCommand,
        ctx: RequestContext,
    ) -> ApiResult<Product> {
        let settings = self.settings_repo.load_settings().await?;
        let prev = self.product_repo.get_by_id(command.id).await?;
        let updated = self
            .product_repo
            .update(
                command.id,
                UpdateProduct {
                    category_id: command.category_id,
                    details: command.details,
                    external_id: command.external_id,
                    fulfillment_image_id: command.fulfillment_image_id,
                    fulfillment_text: command.fulfillment_text,
                    image_id: command.image_id,
                    name: command.name,
                    base_price: command.base_price,
                    subscription_period_days: command.subscription_period_days,
                    r#type: command.r#type,
                },
            )
            .await?;

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::ProductUpdate,
                status: AuditStatus::Success,
                admin_user_id: Some(command.updated_by),
                customer_id: None,
                error_message: None,
                ip_address: ctx.ip_address,
                new_values: serde_json::to_value(updated.clone()).ok(),
                old_values: serde_json::to_value(prev.clone()).ok(),
                request_id: Some(ctx.request_id),
                target_id: prev.id.to_string(),
                target_table: "products".to_string(),
                user_agent: ctx.user_agent.clone(),
            })
            .await?;

        if let Some(stock) = command.stock {
            let current = self
                .stock_movement_repo
                .get_last_by_product_id(updated.id)
                .await
                .map(|r| r.quantity)
                // If initial stock is zero, we don't have any stock movement
                .unwrap_or_default();

            if current == stock {
                return Ok(from_product_row(updated, &settings));
            }
            let stock_movement = self
                .stock_movement_repo
                .create(NewStockMovement {
                    product_id: updated.id,
                    quantity: stock - current,
                    created_by: updated.created_by,
                    r#type: StockMovementType::Adjustment,
                    order_id: None,
                    description: None,
                    reference_id: None,
                })
                .await?;

            self.audit_log_service
                .create(NewAuditLog {
                    action: AuditAction::StockMovementCreate,
                    status: AuditStatus::Success,
                    admin_user_id: Some(command.updated_by),
                    customer_id: None,
                    error_message: None,
                    ip_address: ctx.ip_address,
                    new_values: serde_json::to_value(stock_movement.clone()).ok(),
                    old_values: None,
                    request_id: Some(ctx.request_id),
                    target_id: stock_movement.id.to_string(),
                    target_table: "stock_movements".to_string(),
                    user_agent: ctx.user_agent.clone(),
                })
                .await?;
        }

        Ok(from_product_row(updated, &settings))
    }

    async fn delete(&self, command: DeleteProductCommand) -> ApiResult<()> {
        let prev = self.product_repo.get_by_id(command.id).await?;
        self.product_repo.delete(command.id).await?;

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::ProductDelete,
                status: AuditStatus::Success,
                admin_user_id: Some(command.deleted_by),
                customer_id: None,
                error_message: None,
                new_values: None,
                old_values: serde_json::to_value(prev.clone()).ok(),
                target_id: command.id.to_string(),
                target_table: "products".to_string(),
                ip_address: command.ctx.clone().and_then(|ctx| ctx.ip_address),
                request_id: command.ctx.clone().map(|ctx| ctx.request_id),
                user_agent: command.ctx.and_then(|ctx| ctx.user_agent),
            })
            .await?;

        Ok(())
    }

    async fn get_for_external_provider(
        &self,
        provider_name: &str,
        external_id: &str,
    ) -> ApiResult<Product> {
        let settings = self.settings_repo.load_settings().await?;
        let res = self
            .product_repo
            .get_for_external_provider(provider_name, external_id)
            .await?;
        Ok(from_product_row(res, &settings))
    }

    async fn get_all_external_provider(&self, provider_name: &str) -> ApiResult<Vec<Product>> {
        let settings = self.settings_repo.load_settings().await?;
        let res = self
            .product_repo
            .get_all_external_provider(provider_name)
            .await?;
        Ok(res
            .iter()
            .map(|row| from_product_row(row.clone(), &settings.clone()))
            .collect())
    }

    async fn upload_products(
        &self,
        command: UploadProductsCommand,
    ) -> ApiResult<(Vec<Product>, Vec<String>)> {
        let mut created = Vec::new();
        let mut errors = Vec::new();
        for row in command.products {
            let Some(price) = Decimal::from_f64(row.price) else {
                errors.push(format!(
                    "Product '{}' has invalid price '{}'",
                    row.name, row.price
                ));
                continue;
            };
            let Ok(category) = self
                .category_service
                .create_category_sequence(CreateCategorySequenceCommand {
                    created_by: command.created_by,
                    ctx: command.ctx.clone(),
                    name: row.category.clone(),
                })
                .await
            else {
                errors.push(format!(
                    "Product '{}' has invalid category '{}'",
                    row.name, row.category
                ));
                continue;
            };
            let Some(category) = category else {
                errors.push(format!(
                    "Product '{}' has invalid category '{}'",
                    row.name, row.category
                ));
                continue;
            };
            if self.product_repo.find_by_name(&row.name).await.is_ok() {
                continue;
            }
            match self
                .create(CreateProductCommand {
                    category_id: category.id,
                    initial_stock: Some(row.initial_stock),
                    created_by: command.created_by,
                    details: None,
                    external_id: None,
                    fulfillment_image_id: None,
                    fulfillment_text: None,
                    image_id: None,
                    name: row.name.clone(),
                    base_price: price,
                    provider_name: "internal".to_string(),
                    subscription_period_days: None,
                    r#type: ProductType::Item,
                    ctx: command.ctx.clone(),
                })
                .await
            {
                Ok(product) => created.push(product),
                Err(_) => errors.push(format!("Product '{}' could not be created", row.name)),
            }
        }
        Ok((Vec::new(), Vec::new()))
    }
}

fn from_product_row(res: ProductRow, settings: &Settings) -> Product {
    Product {
        price: calc_product_price(&res.base_price, settings),
        id: res.id,
        name: res.name,
        image_id: res.image_id,
        r#type: res.r#type,
        stock: res.stock,
        subscription_period_days: res.subscription_period_days,
        details: res.details,
        fulfillment_text: res.fulfillment_text,
        fulfillment_image_id: res.fulfillment_image_id,
        base_price: res.base_price,
        provider_name: res.provider_name,
        external_id: res.external_id,
        created_at: res.created_at,
        created_by: res.created_by,
        updated_at: res.updated_at,
        deleted_at: res.deleted_at,
        category_id: res.category_id,
    }
}

fn calc_product_price(base_price: &Decimal, settings: &Settings) -> Decimal {
    let global_markup = settings.pricing_global_markup;
    let gateway_markup = settings.pricing_gateway_markup;
    (base_price * (dec!(1) + global_markup / dec!(100))) / (dec!(1) - gateway_markup / dec!(100))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_settings(global_markup: Decimal, gateway_markup: Decimal) -> Settings {
        Settings {
            pricing_global_markup: global_markup,
            pricing_gateway_markup: gateway_markup,
            bot_messages_new_user_welcome: "".to_string(),
            bot_messages_new_user_welcome_image_id: None,
            bot_messages_returning_user_welcome: "".to_string(),
            bot_messages_returning_user_welcome_image_id: None,
            bot_messages_support: "".to_string(),
            bot_messages_support_image_id: None,
            pricing_gateway_bonus_mock_provider: dec!(0),
            pricing_gateway_bonus_platform_card: dec!(0),
            pricing_gateway_bonus_platform_sbp: dec!(0),
            pricing_platform_commission: dec!(0),
            referral_percentage: dec!(0),
            referral_program_enabled: false,
        }
    }

    #[test]
    fn test_calc_product_price_no_markup() {
        let settings = get_settings(dec!(0), dec!(0));
        let result = calc_product_price(&dec!(100), &settings);
        assert_eq!(result, dec!(100));
    }

    #[test]
    fn test_calc_product_price_only_global_markup() {
        let settings = get_settings(dec!(10), dec!(0));
        let result = calc_product_price(&dec!(100), &settings);
        assert_eq!(result, dec!(110));
    }

    #[test]
    fn test_calc_product_price_only_gateway_markup() {
        let settings = get_settings(dec!(0), dec!(10));
        let result = calc_product_price(&dec!(100), &settings);
        // We need final amount after gateway fee to be 100
        // So: x * (1 - 0.1) = 100 → x = 100 / 0.9 ≈ 111.111...
        assert_eq!(result, dec!(111.11111111111111111111111111));
    }

    #[test]
    fn test_calc_product_price_both_markups() {
        let settings = get_settings(dec!(20), dec!(10));
        let base = dec!(100);
        // Step 1: 100 * 1.2 = 120
        // Step 2: 120 / 0.9 ≈ 133.333...
        let expected = dec!(133.33333333333333333333333333);
        let result = calc_product_price(&base, &settings);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_calc_product_price_with_fractional_base() {
        let settings = get_settings(dec!(5), dec!(2.5));
        let base = dec!(99.99);
        let result = calc_product_price(&base, &settings);
        // 99.99 * 1.05 = 104.9895
        // 104.9895 / (1 - 0.025) = 104.9895 / 0.975 ≈ 107.68153846153846...
        let expected = dec!(107.68153846153846153846153846);
        assert_eq!(result, expected);
    }
}
