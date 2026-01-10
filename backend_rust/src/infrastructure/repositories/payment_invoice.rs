use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::{RepositoryError, RepositoryResult},
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::PaginatedResult,
        payment_invoice::{
            InvoiceStatus, NewPaymentInvoice, PaymentInvoiceListQuery, PaymentInvoiceRow,
            UpdatePaymentInvoice,
        },
    },
};

#[async_trait]
pub trait PaymentInvoiceRepositoryTrait {
    async fn get_list(
        &self,
        query: PaymentInvoiceListQuery,
    ) -> RepositoryResult<PaginatedResult<PaymentInvoiceRow>>;
    async fn create(
        &self,
        payment_invoice: NewPaymentInvoice,
    ) -> RepositoryResult<PaymentInvoiceRow>;
    async fn update(
        &self,
        id: i64,
        payment_invoice: UpdatePaymentInvoice,
    ) -> RepositoryResult<PaymentInvoiceRow>;
    async fn get_by_id(&self, id: i64) -> RepositoryResult<PaymentInvoiceRow>;
}

#[derive(Clone)]
pub struct PaymentInvoiceRepository {
    pool: Arc<PgPool>,
}

impl PaymentInvoiceRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentInvoiceRepositoryTrait for PaymentInvoiceRepository {
    async fn get_list(
        &self,
        query: PaymentInvoiceListQuery,
    ) -> RepositoryResult<PaginatedResult<PaymentInvoiceRow>> {
        let mut count_qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM payment_invoices");
        apply_filters(&mut count_qb, &query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT * FROM payment_invoices");
        apply_list_query(&mut query_builder, &query);
        let query = query_builder.build_query_as::<PaymentInvoiceRow>();
        let items = query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn create(
        &self,
        payment_invoice: NewPaymentInvoice,
    ) -> RepositoryResult<PaymentInvoiceRow> {
        let result = sqlx::query_as!(
            PaymentInvoiceRow,
            r#"
            INSERT INTO payment_invoices (
                customer_id, original_amount, amount, status, expires_at, gateway, gateway_invoice_id,
                order_id, payment_details, bot_message_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING
                id, customer_id, original_amount, amount, status as "status: _", created_at, updated_at,
                expires_at, deleted_at, gateway, gateway_invoice_id, order_id, payment_details,
                bot_message_id, notification_sent_at
            "#,
            payment_invoice.customer_id,
            payment_invoice.original_amount,
            payment_invoice.amount,
            payment_invoice.status as InvoiceStatus,
            payment_invoice.expires_at,
            payment_invoice.gateway as _,
            payment_invoice.gateway_invoice_id,
            payment_invoice.order_id,
            payment_invoice.payment_details,
            payment_invoice.bot_message_id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn update(
        &self,
        id: i64,
        payment_invoice: UpdatePaymentInvoice,
    ) -> RepositoryResult<PaymentInvoiceRow> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("UPDATE payment_invoices SET status = COALESCE(");

        query_builder.push_bind(payment_invoice.status);
        query_builder.push(", status)");

        if let Some(notification_sent_at) = payment_invoice.notification_sent_at {
            query_builder.push(", notification_sent_at = ");
            if let Some(notification_sent_at) = notification_sent_at {
                query_builder.push_bind(notification_sent_at);
            } else {
                query_builder.push("NULL");
            }
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" RETURNING *");

        let query = query_builder.build_query_as::<PaymentInvoiceRow>();

        query
            .fetch_one(&*self.pool)
            .await
            .map_err(RepositoryError::from)
    }

    async fn get_by_id(&self, id: i64) -> RepositoryResult<PaymentInvoiceRow> {
        let result = sqlx::query_as!(
            PaymentInvoiceRow,
            r#"
            SELECT 
                id, customer_id, original_amount, amount, status as "status: _", created_at, updated_at,
                expires_at, deleted_at, gateway, gateway_invoice_id, order_id, payment_details,
                bot_message_id, notification_sent_at
            FROM payment_invoices WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        common::{OrderDir, Pagination},
        payment::PaymentSystem,
    };
    use chrono::{Duration, Utc};
    use rust_decimal::Decimal;
    use serde_json::Value;
    use sqlx::PgPool;
    use uuid::Uuid;

    async fn create_test_customer(pool: &PgPool, telegram_id: i64) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot) VALUES ($1, 1, 1) RETURNING id",
            telegram_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_bot(pool: &PgPool, token: &str, username: &str) -> i64 {
        sqlx::query_scalar!(
            r#"INSERT INTO bots (owner_id, token, username, type, is_active, is_primary, referral_percentage, created_by) VALUES (1, $1, $2, 'main', true, false, 0.1, 1) RETURNING id"#,
            token,
            username
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_order(pool: &PgPool, customer_id: i64, bot_id: i64) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO orders (customer_id, bot_id, status, amount, currency) VALUES ($1, $2, 'created', 10.0, 'USD') RETURNING id",
            customer_id,
            bot_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_get_payment_invoice(pool: PgPool) {
        let repo = PaymentInvoiceRepository::new(Arc::new(pool.clone()));
        let customer_id = create_test_customer(&pool, 123).await;
        let bot_id = create_test_bot(&pool, "invoice_bot_1", "invoice_bot_1").await;
        let _order_id = create_test_order(&pool, customer_id, bot_id).await;
        let expires_at = Utc::now() + Duration::days(1);

        let new_invoice = NewPaymentInvoice {
            customer_id,
            order_id: Uuid::new_v4(),
            original_amount: Decimal::from(100),
            amount: Decimal::from(100),
            status: InvoiceStatus::Pending,
            expires_at,
            gateway: PaymentSystem::Mock,
            gateway_invoice_id: "test_invoice_id".to_string(),
            payment_details: Value::Null,
            bot_message_id: None,
        };

        // Create an invoice
        let created_invoice = repo.create(new_invoice).await.unwrap();
        assert_eq!(created_invoice.customer_id, customer_id);

        // Update the invoice
        let update_data = UpdatePaymentInvoice {
            status: Some(InvoiceStatus::Completed),
            notification_sent_at: Some(Some(Utc::now())),
        };
        let updated_invoice = repo.update(created_invoice.id, update_data).await.unwrap();
        assert_eq!(updated_invoice.status, InvoiceStatus::Completed);
        assert!(updated_invoice.notification_sent_at.is_some());

        // Get the list of invoices
        let query = PaymentInvoiceListQuery {
            filters: vec![],
            pagination: Pagination {
                page: 1,
                page_size: 10,
            },
            order_by: None,
            order_dir: OrderDir::default(),
            _phantom: std::marker::PhantomData,
        };
        let invoices = repo.get_list(query).await.unwrap();
        assert!(!invoices.items.is_empty());
    }
}
