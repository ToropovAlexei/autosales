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
            payment_invoice.gateway,
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
        bot: UpdatePaymentInvoice,
    ) -> RepositoryResult<PaymentInvoiceRow> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("UPDATE payment_invoices SET status = COALESCE(");

        query_builder.push_bind(bot.status);
        query_builder.push(", status)");

        if let Some(notification_sent_at) = bot.notification_sent_at {
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
}
