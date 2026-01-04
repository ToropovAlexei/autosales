use std::sync::Arc;

use async_trait::async_trait;
use bigdecimal::BigDecimal;
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::{
        audit_log::AuditLogRepository,
        products::{ProductRepository, ProductRepositoryTrait},
        stock_movement::{StockMovementRepository, StockMovementRepositoryTrait},
    },
    middlewares::context::RequestContext,
    models::{
        audit_log::{AuditAction, AuditStatus, NewAuditLog},
        common::PaginatedResult,
        product::{NewProduct, ProductListQuery, ProductRow, ProductType, UpdateProduct},
        stock_movement::{NewStockMovement, StockMovementType},
    },
    services::audit_log::{AuditLogService, AuditLogServiceTrait},
};

#[derive(Debug)]
pub struct CreateProductCommand {
    pub name: String,
    pub price: BigDecimal,
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
}

#[derive(Debug)]
pub struct UpdateProductCommand {
    pub id: i64,
    pub name: Option<String>,
    pub price: Option<BigDecimal>,
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
}

#[async_trait]
pub trait ProductServiceTrait: Send + Sync {
    async fn get_list(&self, query: ProductListQuery) -> ApiResult<PaginatedResult<ProductRow>>;
    async fn create(
        &self,
        command: CreateProductCommand,
        ctx: RequestContext,
    ) -> ApiResult<ProductRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<ProductRow>;
    async fn update(
        &self,
        command: UpdateProductCommand,
        ctx: RequestContext,
    ) -> ApiResult<ProductRow>;
    async fn delete(&self, command: DeleteProductCommand, ctx: RequestContext) -> ApiResult<()>;
}

pub struct ProductService<R, S, A> {
    product_repo: Arc<R>,
    stock_movement_repo: Arc<S>,
    audit_log_service: Arc<A>,
}

impl<R, S, A> ProductService<R, S, A>
where
    R: ProductRepositoryTrait + Send + Sync,
    S: StockMovementRepositoryTrait + Send + Sync,
    A: AuditLogServiceTrait + Send + Sync,
{
    pub fn new(
        product_repo: Arc<R>,
        stock_movement_repo: Arc<S>,
        audit_log_service: Arc<A>,
    ) -> Self {
        Self {
            product_repo,
            stock_movement_repo,
            audit_log_service,
        }
    }
}

#[async_trait]
impl ProductServiceTrait
    for ProductService<
        ProductRepository,
        StockMovementRepository,
        AuditLogService<AuditLogRepository>,
    >
{
    async fn get_list(&self, query: ProductListQuery) -> ApiResult<PaginatedResult<ProductRow>> {
        self.product_repo
            .get_list(query)
            .await
            .map_err(ApiError::from)
    }

    async fn create(
        &self,
        command: CreateProductCommand,
        ctx: RequestContext,
    ) -> ApiResult<ProductRow> {
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
                price: command.price,
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
                ip_address: ctx.ip_address,
                new_values: serde_json::to_value(created.clone()).ok(),
                old_values: None,
                request_id: Some(ctx.request_id),
                target_id: created.id.to_string(),
                target_table: "products".to_string(),
                user_agent: ctx.user_agent.clone(),
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
                    ip_address: ctx.ip_address,
                    new_values: serde_json::to_value(stock_movement.clone()).ok(),
                    old_values: None,
                    request_id: Some(ctx.request_id),
                    target_id: created.id.to_string(),
                    target_table: "stock_movements".to_string(),
                    user_agent: ctx.user_agent.clone(),
                })
                .await?;
        }

        Ok(created)
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<ProductRow> {
        let res = self.product_repo.get_by_id(id).await?;
        Ok(res)
    }

    async fn update(
        &self,
        command: UpdateProductCommand,
        ctx: RequestContext,
    ) -> ApiResult<ProductRow> {
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
                    price: command.price,
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
                return Ok(updated);
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

        Ok(updated)
    }

    async fn delete(&self, command: DeleteProductCommand, ctx: RequestContext) -> ApiResult<()> {
        let prev = self.product_repo.get_by_id(command.id).await?;
        self.product_repo.delete(command.id).await?;

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::ProductDelete,
                status: AuditStatus::Success,
                admin_user_id: Some(command.deleted_by),
                customer_id: None,
                error_message: None,
                ip_address: ctx.ip_address,
                new_values: None,
                old_values: serde_json::to_value(prev.clone()).ok(),
                request_id: Some(ctx.request_id),
                target_id: command.id.to_string(),
                target_table: "products".to_string(),
                user_agent: ctx.user_agent.clone(),
            })
            .await?;

        Ok(())
    }
}
