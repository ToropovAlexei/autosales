use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use shared_dtos::invoice::InvoiceStatus;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::{
    errors::repository::{RepositoryError, RepositoryResult},
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::PaginatedResult,
        payment_invoice::{
            NewPaymentInvoice, PaymentInvoiceListQuery, PaymentInvoiceRow, UpdatePaymentInvoice,
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
    async fn get_by_order_id(&self, order_id: Uuid) -> RepositoryResult<PaymentInvoiceRow>;
    async fn get_for_customer(&self, customer_id: i64) -> RepositoryResult<Vec<PaymentInvoiceRow>>;
    async fn expire_old_invoices(&self) -> RepositoryResult<u64>;
    async fn get_pending_invoices(
        &self,
        older_than: DateTime<Utc>,
    ) -> RepositoryResult<Vec<PaymentInvoiceRow>>;
    async fn mark_invoices_notified(&self, ids: &[i64]) -> RepositoryResult<u64>;
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
                expires_at, deleted_at, gateway as "gateway: _", gateway_invoice_id, order_id, payment_details,
                bot_message_id, notification_sent_at, receipt_requested_at,
                receipt_submitted_at, dispute_opened_at, finished_at
            "#,
            payment_invoice.customer_id,
            payment_invoice.original_amount,
            payment_invoice.amount,
            payment_invoice.status as InvoiceStatus,
            payment_invoice.expires_at,
            payment_invoice.gateway as _,
            payment_invoice.gateway_invoice_id,
            payment_invoice.order_id,
            payment_invoice.payment_details.map(|p| serde_json::to_value(p).unwrap_or_default()).unwrap_or_default(),
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

        if let Some(receipt_requested_at) = payment_invoice.receipt_requested_at {
            query_builder.push(", receipt_requested_at = ");
            query_builder.push_bind(receipt_requested_at);
        }
        if let Some(receipt_submitted_at) = payment_invoice.receipt_submitted_at {
            query_builder.push(", receipt_submitted_at = ");
            query_builder.push_bind(receipt_submitted_at);
        }
        if let Some(dispute_opened_at) = payment_invoice.dispute_opened_at {
            query_builder.push(", dispute_opened_at = ");
            query_builder.push_bind(dispute_opened_at);
        }
        if let Some(finished_at) = payment_invoice.finished_at {
            query_builder.push(", finished_at = ");
            query_builder.push_bind(finished_at);
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
                expires_at, deleted_at, gateway as "gateway: _", gateway_invoice_id, order_id, payment_details,
                bot_message_id, notification_sent_at, receipt_requested_at,
                receipt_submitted_at, dispute_opened_at, finished_at
            FROM payment_invoices WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn get_by_order_id(&self, order_id: Uuid) -> RepositoryResult<PaymentInvoiceRow> {
        let result = sqlx::query_as!(
            PaymentInvoiceRow,
            r#"
            SELECT
                id, customer_id, original_amount, amount, status as "status: _", created_at, updated_at,
                expires_at, deleted_at, gateway as "gateway: _", gateway_invoice_id, order_id, payment_details,
                bot_message_id, notification_sent_at, receipt_requested_at,
                receipt_submitted_at, dispute_opened_at, finished_at
            FROM payment_invoices WHERE order_id = $1"#,
            order_id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn get_for_customer(&self, customer_id: i64) -> RepositoryResult<Vec<PaymentInvoiceRow>> {
        let result = sqlx::query_as!(
            PaymentInvoiceRow,
            r#"
            SELECT
                id, customer_id, original_amount, amount, status as "status: _", created_at, updated_at,
                expires_at, deleted_at, gateway as "gateway: _", gateway_invoice_id, order_id, payment_details,
                bot_message_id, notification_sent_at, receipt_requested_at,
                receipt_submitted_at, dispute_opened_at, finished_at
            FROM payment_invoices WHERE customer_id = $1"#,
            customer_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn expire_old_invoices(&self) -> RepositoryResult<u64> {
        let result = sqlx::query!(
            r#"
        UPDATE payment_invoices
        SET status = 'expired'
        WHERE status = 'pending'
        AND expires_at < NOW()
        AND deleted_at IS NULL
        "#,
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn get_pending_invoices(
        &self,
        older_than: DateTime<Utc>,
    ) -> RepositoryResult<Vec<PaymentInvoiceRow>> {
        let result = sqlx::query_as!(
            PaymentInvoiceRow,
            r#"
            SELECT
                id, customer_id, original_amount, amount, status as "status: _", created_at, updated_at,
                expires_at, deleted_at, gateway as "gateway: _", gateway_invoice_id, order_id, payment_details,
                bot_message_id, notification_sent_at, receipt_requested_at,
                receipt_submitted_at, dispute_opened_at, finished_at
            FROM payment_invoices
            WHERE
                status IN ('pending', 'processing', 'awaiting_receipt', 'receipt_submitted', 'disputed') AND
                created_at < $1 AND
                deleted_at IS NULL
            "#,
            older_than
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn mark_invoices_notified(&self, ids: &[i64]) -> RepositoryResult<u64> {
        let result = sqlx::query!(
            "UPDATE payment_invoices SET notification_sent_at = NOW() WHERE id = ANY($1)",
            ids
        )
        .execute(&*self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use rust_decimal::Decimal;
    use shared_dtos::invoice::PaymentSystem;
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

    async fn create_test_bot(
        pool: &PgPool,
        owner_id: Option<i64>,
        token: &str,
        username: &str,
    ) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO bots (
                owner_id, token, username, type, is_active, is_primary, referral_percentage, created_by
            )
            VALUES ($1, $2, $3, 'main', true, false, 0.1, 1)
            RETURNING id
            "#,
            owner_id,
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

    async fn create_test_invoice(
        pool: &PgPool,
        customer_id: i64,
        status: InvoiceStatus,
        gateway_invoice_id: &str,
        created_at: DateTime<Utc>,
    ) -> PaymentInvoiceRow {
        let new_invoice = NewPaymentInvoice {
            customer_id,
            order_id: Uuid::new_v4(),
            original_amount: Decimal::from(10),
            amount: Decimal::from(10),
            status,
            expires_at: Utc::now() + Duration::days(1),
            gateway: PaymentSystem::Mock,
            gateway_invoice_id: gateway_invoice_id.to_string(),
            payment_details: None,
            bot_message_id: None,
        };

        sqlx::query_as!(
            PaymentInvoiceRow,
            r#"
            INSERT INTO payment_invoices (
                customer_id, original_amount, amount, status, expires_at, gateway, gateway_invoice_id,
                order_id, payment_details, bot_message_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING
                id, customer_id, original_amount, amount, status as "status: _", created_at, updated_at,
                expires_at, deleted_at, gateway as "gateway: _", gateway_invoice_id, order_id, payment_details,
                bot_message_id, notification_sent_at, receipt_requested_at,
                receipt_submitted_at, dispute_opened_at, finished_at
            "#,
            new_invoice.customer_id,
            new_invoice.original_amount,
            new_invoice.amount,
            new_invoice.status as InvoiceStatus,
            new_invoice.expires_at,
            new_invoice.gateway as _,
            new_invoice.gateway_invoice_id,
            new_invoice.order_id,
            new_invoice
                .payment_details
                .map(|p| serde_json::to_value(p).unwrap_or_default())
                .unwrap_or_default(),
            new_invoice.bot_message_id,
            created_at
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_get_payment_invoice(pool: PgPool) {
        let repo = PaymentInvoiceRepository::new(Arc::new(pool.clone()));
        let customer_id = create_test_customer(&pool, 123).await;
        let bot_id = create_test_bot(&pool, None, "invoice_bot_1", "invoice_bot_1").await;
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
            payment_details: None,
            bot_message_id: None,
        };

        // Create an invoice
        let created_invoice = repo.create(new_invoice).await.unwrap();
        assert_eq!(created_invoice.customer_id, customer_id);

        // Update the invoice
        let update_data = UpdatePaymentInvoice {
            status: Some(InvoiceStatus::Completed),
            notification_sent_at: Some(Some(Utc::now())),
            ..Default::default()
        };
        let updated_invoice = repo.update(created_invoice.id, update_data).await.unwrap();
        assert_eq!(updated_invoice.status, InvoiceStatus::Completed);
        assert!(updated_invoice.notification_sent_at.is_some());

        // Get the list of invoices
        let query = PaymentInvoiceListQuery::default();
        let invoices = repo.get_list(query).await.unwrap();
        assert!(!invoices.items.is_empty());
    }

    #[sqlx::test]
    async fn test_get_by_id(pool: PgPool) {
        let repo = PaymentInvoiceRepository::new(Arc::new(pool.clone()));
        let customer_id = create_test_customer(&pool, 456).await;
        let expires_at = Utc::now() + Duration::days(1);

        let new_invoice = NewPaymentInvoice {
            customer_id,
            order_id: Uuid::new_v4(),
            original_amount: Decimal::from(120),
            amount: Decimal::from(120),
            status: InvoiceStatus::Pending,
            expires_at,
            gateway: PaymentSystem::Mock,
            gateway_invoice_id: "test_get_by_id_invoice".to_string(),
            payment_details: None,
            bot_message_id: None,
        };
        let created_invoice = repo.create(new_invoice).await.unwrap();

        let fetched_invoice = repo.get_by_id(created_invoice.id).await.unwrap();

        assert_eq!(fetched_invoice.id, created_invoice.id);
        assert_eq!(
            fetched_invoice.gateway_invoice_id,
            "test_get_by_id_invoice".to_string()
        );
    }

    #[sqlx::test]
    async fn test_get_by_order_id(pool: PgPool) {
        let repo = PaymentInvoiceRepository::new(Arc::new(pool.clone()));
        let customer_id = create_test_customer(&pool, 789).await;
        let order_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::days(1);

        let new_invoice = NewPaymentInvoice {
            customer_id,
            order_id,
            original_amount: Decimal::from(150),
            amount: Decimal::from(150),
            status: InvoiceStatus::Pending,
            expires_at,
            gateway: PaymentSystem::Mock,
            gateway_invoice_id: "test_get_by_order_id_invoice".to_string(),
            payment_details: None,
            bot_message_id: None,
        };
        let created_invoice = repo.create(new_invoice).await.unwrap();

        let fetched_invoice = repo.get_by_order_id(order_id).await.unwrap();

        assert_eq!(fetched_invoice.id, created_invoice.id);
        assert_eq!(fetched_invoice.order_id, order_id);
    }

    #[sqlx::test]
    async fn test_get_for_customer(pool: PgPool) {
        let repo = PaymentInvoiceRepository::new(Arc::new(pool.clone()));
        let customer_id_1 = create_test_customer(&pool, 1001).await;
        let customer_id_2 = create_test_customer(&pool, 1002).await;
        let expires_at = Utc::now() + Duration::days(1);

        // Create invoices for customer 1
        let new_invoice_1_1 = NewPaymentInvoice {
            customer_id: customer_id_1,
            order_id: Uuid::new_v4(),
            original_amount: Decimal::from(50),
            amount: Decimal::from(50),
            status: InvoiceStatus::Pending,
            expires_at,
            gateway: PaymentSystem::Mock,
            gateway_invoice_id: "cust1_invoice1".to_string(),
            payment_details: None,
            bot_message_id: None,
        };
        repo.create(new_invoice_1_1).await.unwrap();

        let new_invoice_1_2 = NewPaymentInvoice {
            customer_id: customer_id_1,
            order_id: Uuid::new_v4(),
            original_amount: Decimal::from(75),
            amount: Decimal::from(75),
            status: InvoiceStatus::Completed,
            expires_at,
            gateway: PaymentSystem::Mock,
            gateway_invoice_id: "cust1_invoice2".to_string(),
            payment_details: None,
            bot_message_id: None,
        };
        repo.create(new_invoice_1_2).await.unwrap();

        // Create an invoice for customer 2
        let new_invoice_2_1 = NewPaymentInvoice {
            customer_id: customer_id_2,
            order_id: Uuid::new_v4(),
            original_amount: Decimal::from(100),
            amount: Decimal::from(100),
            status: InvoiceStatus::Pending,
            expires_at,
            gateway: PaymentSystem::Mock,
            gateway_invoice_id: "cust2_invoice1".to_string(),
            payment_details: None,
            bot_message_id: None,
        };
        repo.create(new_invoice_2_1).await.unwrap();

        let customer_1_invoices = repo.get_for_customer(customer_id_1).await.unwrap();
        assert_eq!(customer_1_invoices.len(), 2);
        assert!(
            customer_1_invoices
                .iter()
                .all(|inv| inv.customer_id == customer_id_1)
        );

        let customer_2_invoices = repo.get_for_customer(customer_id_2).await.unwrap();
        assert_eq!(customer_2_invoices.len(), 1);
        assert!(
            customer_2_invoices
                .iter()
                .all(|inv| inv.customer_id == customer_id_2)
        );
    }

    #[sqlx::test]
    async fn test_expire_old_invoices(pool: PgPool) {
        let repo = PaymentInvoiceRepository::new(Arc::new(pool.clone()));
        let customer_id = create_test_customer(&pool, 1003).await;

        // Create an invoice that has expired
        let expired_invoice_date = Utc::now() - Duration::hours(1);
        let new_expired_invoice = NewPaymentInvoice {
            customer_id,
            order_id: Uuid::new_v4(),
            original_amount: Decimal::from(200),
            amount: Decimal::from(200),
            status: InvoiceStatus::Pending,
            expires_at: expired_invoice_date,
            gateway: PaymentSystem::Mock,
            gateway_invoice_id: "expired_invoice_id".to_string(),
            payment_details: None,
            bot_message_id: None,
        };
        let created_expired_invoice = repo.create(new_expired_invoice).await.unwrap();

        // Create a pending invoice that has not expired
        let non_expired_invoice_date = Utc::now() + Duration::hours(1);
        let new_non_expired_invoice = NewPaymentInvoice {
            customer_id,
            order_id: Uuid::new_v4(),
            original_amount: Decimal::from(50),
            amount: Decimal::from(50),
            status: InvoiceStatus::Pending,
            expires_at: non_expired_invoice_date,
            gateway: PaymentSystem::Mock,
            gateway_invoice_id: "non_expired_invoice_id".to_string(),
            payment_details: None,
            bot_message_id: None,
        };
        let created_non_expired_invoice = repo.create(new_non_expired_invoice).await.unwrap();

        // Call the method to expire old invoices
        let rows_affected = repo.expire_old_invoices().await.unwrap();
        assert_eq!(rows_affected, 1);

        // Verify the status of the expired invoice
        let updated_expired_invoice = repo.get_by_id(created_expired_invoice.id).await.unwrap();
        assert_eq!(updated_expired_invoice.status, InvoiceStatus::Expired);

        // Verify the status of the non-expired invoice remains unchanged
        let updated_non_expired_invoice = repo
            .get_by_id(created_non_expired_invoice.id)
            .await
            .unwrap();
        assert_eq!(updated_non_expired_invoice.status, InvoiceStatus::Pending);
    }

    #[sqlx::test]
    async fn test_get_pending_invoices(pool: PgPool) {
        let repo = PaymentInvoiceRepository::new(Arc::new(pool.clone()));
        let customer_id = create_test_customer(&pool, 1004).await;

        // Invoice 1: Pending, older than 2 hours (should be returned)
        create_test_invoice(
            &pool,
            customer_id,
            InvoiceStatus::Pending,
            "pending_old_invoice",
            Utc::now() - Duration::hours(3),
        )
        .await;

        // Invoice 2: Completed, older than 2 hours (should NOT be returned)
        create_test_invoice(
            &pool,
            customer_id,
            InvoiceStatus::Completed,
            "completed_old_invoice",
            Utc::now() - Duration::hours(3),
        )
        .await;

        // Invoice 3: Pending, newer than 2 hours (should NOT be returned)
        create_test_invoice(
            &pool,
            customer_id,
            InvoiceStatus::Pending,
            "pending_new_invoice",
            Utc::now() - Duration::hours(1),
        )
        .await;

        // Invoice 4: Processing, older than 2 hours (should be returned)
        create_test_invoice(
            &pool,
            customer_id,
            InvoiceStatus::Processing,
            "processing_old_invoice",
            Utc::now() - Duration::hours(4),
        )
        .await;

        let older_than = Utc::now() - Duration::hours(2);
        let pending_invoices = repo.get_pending_invoices(older_than).await.unwrap();

        assert_eq!(pending_invoices.len(), 2);
        assert!(
            pending_invoices
                .iter()
                .any(|inv| inv.gateway_invoice_id == "pending_old_invoice")
        );
        assert!(
            pending_invoices
                .iter()
                .any(|inv| inv.gateway_invoice_id == "processing_old_invoice")
        );
        assert!(
            !pending_invoices
                .iter()
                .any(|inv| inv.gateway_invoice_id == "completed_old_invoice")
        );
        assert!(
            !pending_invoices
                .iter()
                .any(|inv| inv.gateway_invoice_id == "pending_new_invoice")
        );
    }

    #[sqlx::test]
    async fn test_mark_invoices_notified(pool: PgPool) {
        let repo = PaymentInvoiceRepository::new(Arc::new(pool.clone()));
        let customer_id = create_test_customer(&pool, 1005).await;
        let expires_at = Utc::now() + Duration::days(1);

        // Create invoices
        let new_invoice_1 = NewPaymentInvoice {
            customer_id,
            order_id: Uuid::new_v4(),
            original_amount: Decimal::from(10),
            amount: Decimal::from(10),
            status: InvoiceStatus::Pending,
            expires_at,
            gateway: PaymentSystem::Mock,
            gateway_invoice_id: "notify_invoice_1".to_string(),
            payment_details: None,
            bot_message_id: None,
        };
        let created_invoice_1 = repo.create(new_invoice_1).await.unwrap();

        let new_invoice_2 = NewPaymentInvoice {
            customer_id,
            order_id: Uuid::new_v4(),
            original_amount: Decimal::from(20),
            amount: Decimal::from(20),
            status: InvoiceStatus::Pending,
            expires_at,
            gateway: PaymentSystem::Mock,
            gateway_invoice_id: "notify_invoice_2".to_string(),
            payment_details: None,
            bot_message_id: None,
        };
        let created_invoice_2 = repo.create(new_invoice_2).await.unwrap();

        let new_invoice_3 = NewPaymentInvoice {
            customer_id,
            order_id: Uuid::new_v4(),
            original_amount: Decimal::from(30),
            amount: Decimal::from(30),
            status: InvoiceStatus::Completed, // Should not be notified by this method
            expires_at,
            gateway: PaymentSystem::Mock,
            gateway_invoice_id: "notify_invoice_3".to_string(),
            payment_details: None,
            bot_message_id: None,
        };
        let created_invoice_3 = repo.create(new_invoice_3).await.unwrap();

        // Mark invoices 1 and 2 as notified
        let ids_to_notify = vec![created_invoice_1.id, created_invoice_2.id];
        let rows_affected = repo.mark_invoices_notified(&ids_to_notify).await.unwrap();
        assert_eq!(rows_affected, 2);

        // Verify invoice 1
        let invoice_1_updated = repo.get_by_id(created_invoice_1.id).await.unwrap();
        assert!(invoice_1_updated.notification_sent_at.is_some());

        // Verify invoice 2
        let invoice_2_updated = repo.get_by_id(created_invoice_2.id).await.unwrap();
        assert!(invoice_2_updated.notification_sent_at.is_some());

        // Verify invoice 3 (should not be updated)
        let invoice_3_updated = repo.get_by_id(created_invoice_3.id).await.unwrap();
        assert!(invoice_3_updated.notification_sent_at.is_none());
    }
}
