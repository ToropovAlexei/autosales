use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, QueryBuilder};

use crate::{
    errors::repository::{RepositoryError, RepositoryResult},
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        broadcast::{
            BroadcastListQuery, BroadcastRow, BroadcastStatus, NewBroadcast, UpdateBroadcast,
        },
        common::PaginatedResult,
    },
};

#[async_trait]
pub trait BroadcastRepositoryTrait {
    async fn get_list(
        &self,
        query: BroadcastListQuery,
    ) -> RepositoryResult<PaginatedResult<BroadcastRow>>;
    async fn create(&self, broadcast: NewBroadcast) -> RepositoryResult<BroadcastRow>;
    async fn update(&self, id: i64, broadcast: UpdateBroadcast) -> RepositoryResult<BroadcastRow>;
    async fn get_by_id(&self, id: i64) -> RepositoryResult<BroadcastRow>;
    async fn get_ready_broadcasts(&self) -> RepositoryResult<Vec<BroadcastRow>>;
}

#[derive(Clone)]
pub struct BroadcastRepository {
    pool: Arc<PgPool>,
}

impl BroadcastRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BroadcastRepositoryTrait for BroadcastRepository {
    async fn get_list(
        &self,
        query: BroadcastListQuery,
    ) -> RepositoryResult<PaginatedResult<BroadcastRow>> {
        let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM broadcasts");
        apply_filters(&mut count_qb, &query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder = QueryBuilder::new("SELECT * FROM broadcasts");
        apply_list_query(&mut query_builder, &query);
        let items_query = query_builder.build_query_as::<BroadcastRow>();
        let items = items_query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn create(&self, broadcast: NewBroadcast) -> RepositoryResult<BroadcastRow> {
        let result = sqlx::query_as!(
            BroadcastRow,
            r#"
            INSERT INTO broadcasts (status, content_text, content_image_id, filters, created_by, scheduled_for)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING 
                id, status as "status: _", content_text, content_image_id, filters, statistics,
                created_by, scheduled_for, started_at, finished_at, created_at, updated_at
            "#,
            broadcast.status as BroadcastStatus,
            broadcast.content_text,
            broadcast.content_image_id,
            broadcast.filters,
            broadcast.created_by,
            broadcast.scheduled_for
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, id: i64, broadcast: UpdateBroadcast) -> RepositoryResult<BroadcastRow> {
        let mut query_builder = QueryBuilder::new("UPDATE broadcasts SET status = COALESCE(");

        query_builder.push_bind(broadcast.status);
        query_builder.push(", status)");

        if let Some(content_image_id) = broadcast.content_image_id {
            query_builder.push(", content_image_id = ");
            if let Some(content_image_id) = content_image_id {
                query_builder.push_bind(content_image_id);
            } else {
                query_builder.push("NULL");
            }
        }

        if let Some(filters) = broadcast.filters {
            query_builder.push(", filters = ");
            if let Some(filters) = filters {
                query_builder.push_bind(filters);
            } else {
                query_builder.push("NULL");
            }
        }

        if let Some(content_text) = broadcast.content_text {
            query_builder.push(", content_text = ");
            if let Some(content_text) = content_text {
                query_builder.push_bind(content_text);
            } else {
                query_builder.push("NULL");
            }
        }

        if let Some(statistics) = broadcast.statistics {
            query_builder.push(", statistics = ");
            if let Some(statistics) = statistics {
                query_builder.push_bind(statistics);
            } else {
                query_builder.push("NULL");
            }
        }

        if let Some(started_at) = broadcast.started_at {
            query_builder.push(", started_at = ");
            if let Some(started_at) = started_at {
                query_builder.push_bind(started_at);
            } else {
                query_builder.push("NULL");
            }
        }

        if let Some(finished_at) = broadcast.finished_at {
            query_builder.push(", finished_at = ");
            if let Some(finished_at) = finished_at {
                query_builder.push_bind(finished_at);
            } else {
                query_builder.push("NULL");
            }
        }

        if let Some(scheduled_for) = broadcast.scheduled_for {
            query_builder.push(", scheduled_for = ");
            if let Some(scheduled_for) = scheduled_for {
                query_builder.push_bind(scheduled_for);
            } else {
                query_builder.push("NULL");
            }
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" RETURNING *");

        let query = query_builder.build_query_as::<BroadcastRow>();

        query
            .fetch_one(&*self.pool)
            .await
            .map_err(RepositoryError::from)
    }

    async fn get_by_id(&self, id: i64) -> RepositoryResult<BroadcastRow> {
        sqlx::query_as!(
            BroadcastRow,
            r#"
            SELECT  
                id, status as "status: _", content_text, content_image_id, filters, statistics,
                created_by, scheduled_for, started_at, finished_at, created_at, updated_at
            FROM broadcasts WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    async fn get_ready_broadcasts(&self) -> RepositoryResult<Vec<BroadcastRow>> {
        sqlx::query_as!(
            BroadcastRow,
            r#"
            SELECT  
                id, status as "status: _", content_text, content_image_id, filters, statistics,
                created_by, scheduled_for, started_at, finished_at, created_at, updated_at
            FROM broadcasts WHERE
                status = $1
                OR (
                    status = $2
                    AND scheduled_for <= NOW()
                )"#,
            BroadcastStatus::Pending as _,
            BroadcastStatus::Scheduled as _
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::admin_user::AdminUserRow;
    use crate::models::{
        broadcast::{BroadcastListQuery, BroadcastStatus, NewBroadcast, UpdateBroadcast},
        common::{Filter, ScalarValue},
    };
    use chrono::{Duration, Utc};
    use serde_json::json;
    use sqlx::PgPool;

    async fn create_test_admin_user(pool: &PgPool, login: &str) -> AdminUserRow {
        sqlx::query_as!(
            AdminUserRow,
            r#"
            INSERT INTO admin_users (login, hashed_password, two_fa_secret, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            login,
            "password",
            "",
            1i64 // System user
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_broadcast(
        pool: &PgPool,
        created_by: i64,
        status: BroadcastStatus,
        scheduled_for: Option<chrono::DateTime<Utc>>,
    ) -> BroadcastRow {
        let new_broadcast = NewBroadcast {
            status,
            content_text: Some("Test content".to_string()),
            content_image_id: None,
            filters: Some(json!({"filters": [{"field": "balance", "op": "gt", "value": 100}]})),
            created_by,
            scheduled_for,
        };
        sqlx::query_as!(
            BroadcastRow,
            r#"
            INSERT INTO broadcasts (status, content_text, content_image_id, filters, created_by, scheduled_for)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING 
                id, status as "status: _", content_text, content_image_id, filters, statistics,
                created_by, scheduled_for, started_at, finished_at, created_at, updated_at
            "#,
            new_broadcast.status as _,
            new_broadcast.content_text,
            new_broadcast.content_image_id,
            new_broadcast.filters,
            new_broadcast.created_by,
            new_broadcast.scheduled_for
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_get_broadcast(pool: PgPool) {
        let repo = BroadcastRepository::new(Arc::new(pool.clone()));
        let admin_user = create_test_admin_user(&pool, "test_admin_1").await;

        let new_broadcast = NewBroadcast {
            status: BroadcastStatus::Pending,
            content_text: Some("Initial content".to_string()),
            content_image_id: None,
            filters: None,
            created_by: admin_user.id,
            scheduled_for: None,
        };

        let created_broadcast = repo.create(new_broadcast).await.unwrap();
        assert_eq!(created_broadcast.created_by, admin_user.id);
        assert_eq!(created_broadcast.status, BroadcastStatus::Pending);

        let fetched_broadcast = repo.get_by_id(created_broadcast.id).await.unwrap();
        assert_eq!(fetched_broadcast.id, created_broadcast.id);
        assert_eq!(
            fetched_broadcast.content_text,
            Some("Initial content".to_string())
        );
    }

    #[sqlx::test]
    async fn test_update_broadcast(pool: PgPool) {
        let repo = BroadcastRepository::new(Arc::new(pool.clone()));
        let admin_user = create_test_admin_user(&pool, "test_admin_2").await;
        let broadcast =
            create_test_broadcast(&pool, admin_user.id, BroadcastStatus::Pending, None).await;

        let update_payload = UpdateBroadcast {
            status: Some(BroadcastStatus::Completed),
            content_text: Some(Some("Updated content".to_string())),
            content_image_id: None,
            filters: Some(Some(json!({"filters": "updated"}))),
            scheduled_for: None,
            finished_at: Some(Some(Utc::now())),
            started_at: Some(Some(Utc::now())),
            statistics: Some(Some(json!({"statistics": "updated"}))),
        };

        let updated_broadcast = repo.update(broadcast.id, update_payload).await.unwrap();
        assert_eq!(updated_broadcast.status, BroadcastStatus::Completed);
        assert_eq!(
            updated_broadcast.content_text,
            Some("Updated content".to_string())
        );
        assert_eq!(
            updated_broadcast.filters,
            Some(json!({"filters": "updated"}))
        );
    }

    #[sqlx::test]
    async fn test_get_list_broadcasts(pool: PgPool) {
        let repo = BroadcastRepository::new(Arc::new(pool.clone()));
        let admin_user = create_test_admin_user(&pool, "test_admin_3").await;

        create_test_broadcast(&pool, admin_user.id, BroadcastStatus::Pending, None).await;
        create_test_broadcast(&pool, admin_user.id, BroadcastStatus::Completed, None).await;
        create_test_broadcast(&pool, admin_user.id, BroadcastStatus::Completed, None).await;

        let mut query = BroadcastListQuery::default();

        let all_broadcasts = repo.get_list(query.clone()).await.unwrap();
        assert!(all_broadcasts.total >= 3);

        use crate::models::broadcast::BroadcastFilterFields;
        query.filters = vec![Filter {
            field: BroadcastFilterFields::Status,
            op: crate::models::common::Operator::Eq,
            value: crate::models::common::FilterValue::Scalar(ScalarValue::Text(
                "completed".to_string(),
            )),
        }];

        let completed_broadcasts = repo.get_list(query).await.unwrap();
        assert_eq!(completed_broadcasts.items.len(), 2);
        assert_eq!(completed_broadcasts.total, 2);
        assert!(
            completed_broadcasts
                .items
                .iter()
                .all(|b| b.status == BroadcastStatus::Completed)
        );
    }

    #[sqlx::test]
    async fn test_get_ready_broadcasts(pool: PgPool) {
        let repo = BroadcastRepository::new(Arc::new(pool.clone()));
        let admin_user = create_test_admin_user(&pool, "test_admin_4").await;

        // Should be fetched
        create_test_broadcast(&pool, admin_user.id, BroadcastStatus::Pending, None).await;

        // Should be fetched (scheduled in the past)
        create_test_broadcast(
            &pool,
            admin_user.id,
            BroadcastStatus::Scheduled,
            Some(Utc::now() - Duration::minutes(5)),
        )
        .await;

        // Should NOT be fetched (scheduled in the future)
        create_test_broadcast(
            &pool,
            admin_user.id,
            BroadcastStatus::Scheduled,
            Some(Utc::now() + Duration::minutes(5)),
        )
        .await;

        // Should NOT be fetched (already completed)
        create_test_broadcast(&pool, admin_user.id, BroadcastStatus::Completed, None).await;

        // Should NOT be fetched (in progress)
        create_test_broadcast(&pool, admin_user.id, BroadcastStatus::InProgress, None).await;

        let ready_broadcasts = repo.get_ready_broadcasts().await.unwrap();

        assert_eq!(ready_broadcasts.len(), 2);
        assert!(
            ready_broadcasts
                .iter()
                .all(|b| b.status == BroadcastStatus::Pending
                    || (b.status == BroadcastStatus::Scheduled
                        && b.scheduled_for.unwrap() < Utc::now()))
        );
    }
}
