use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::{RepositoryError, RepositoryResult},
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        bot::{BotListQuery, BotRow, BotType, NewBot, UpdateBot},
        common::PaginatedResult,
    },
};

#[async_trait]
pub trait BotRepositoryTrait {
    async fn get_list(&self, query: BotListQuery) -> RepositoryResult<PaginatedResult<BotRow>>;
    async fn create(&self, bot: NewBot) -> RepositoryResult<BotRow>;
    async fn get_by_id(&self, id: i64) -> RepositoryResult<BotRow>;
    async fn get_by_token(&self, token: String) -> RepositoryResult<BotRow>;
    async fn update(&self, id: i64, bot: UpdateBot) -> RepositoryResult<BotRow>;
    async fn set_primary_bot_for_owner(
        &self,
        id: i64,
        owner_id: Option<i64>,
    ) -> RepositoryResult<()>;
    async fn get_primary_bots(&self) -> RepositoryResult<Vec<BotRow>>;
}

#[derive(Clone)]
pub struct BotRepository {
    pool: Arc<PgPool>,
}

impl BotRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BotRepositoryTrait for BotRepository {
    async fn get_list(&self, query: BotListQuery) -> RepositoryResult<PaginatedResult<BotRow>> {
        let mut count_qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT COUNT(*) FROM bots");
        apply_filters(&mut count_qb, &query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM bots");
        apply_list_query(&mut query_builder, &query);
        let query = query_builder.build_query_as::<BotRow>();
        let items = query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn create(&self, bot: NewBot) -> RepositoryResult<BotRow> {
        let result = sqlx::query_as!(
            BotRow,
            r#"
            INSERT INTO bots (
                owner_id, token, username, type, is_active,
                is_primary, referral_percentage, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id, owner_id, token, username, type as "type: _", is_active,
                is_primary, referral_percentage, 
                created_at, updated_at, created_by
            "#,
            bot.owner_id,
            bot.token,
            bot.username,
            bot.r#type as BotType,
            bot.is_active,
            bot.is_primary,
            bot.referral_percentage,
            bot.created_by
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn get_by_id(&self, id: i64) -> RepositoryResult<BotRow> {
        let result = sqlx::query_as!(
            BotRow,
            r#"
        SELECT 
            id, owner_id, token, username, type as "type: _", is_active,
            is_primary, referral_percentage, 
            created_at, updated_at, created_by
        FROM bots WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, id: i64, bot: UpdateBot) -> RepositoryResult<BotRow> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("UPDATE bots SET username = COALESCE(");

        query_builder.push_bind(bot.username);
        query_builder.push(", username)");

        if let Some(is_active) = bot.is_active {
            query_builder.push(", is_active = ");
            query_builder.push_bind(is_active);
        }

        if let Some(is_primary) = bot.is_primary {
            query_builder.push(", is_primary = ");
            query_builder.push_bind(is_primary);
        }

        if let Some(referral_percentage) = bot.referral_percentage {
            query_builder.push(", referral_percentage = ");
            query_builder.push_bind(referral_percentage);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" RETURNING *");

        let query = query_builder.build_query_as::<BotRow>();

        query
            .fetch_one(&*self.pool)
            .await
            .map_err(RepositoryError::from)
    }

    async fn get_by_token(&self, token: String) -> RepositoryResult<BotRow> {
        let result = sqlx::query_as!(
            BotRow,
            r#"
        SELECT 
            id, owner_id, token, username, type as "type: _", is_active,
            is_primary, referral_percentage, 
            created_at, updated_at, created_by
        FROM bots WHERE token = $1"#,
            token
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn set_primary_bot_for_owner(
        &self,
        id: i64,
        owner_id: Option<i64>,
    ) -> RepositoryResult<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "UPDATE bots SET is_primary = false WHERE owner_id IS NOT DISTINCT FROM $1",
            owner_id
        )
        .execute(tx.as_mut())
        .await?;
        sqlx::query!("UPDATE bots SET is_primary = true WHERE id = $1", id)
            .execute(tx.as_mut())
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn get_primary_bots(&self) -> RepositoryResult<Vec<BotRow>> {
        let result = sqlx::query_as!(
            BotRow,
            r#"
        SELECT 
            id, owner_id, token, username, type as "type: _", is_active,
            is_primary, referral_percentage, 
            created_at, updated_at, created_by
        FROM bots WHERE is_primary = true"#,
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        bot::BotType,
        common::{OrderDir, Pagination},
    };
    use rust_decimal::Decimal;
    use sqlx::PgPool;
    use std::str::FromStr;

    async fn create_test_bot(pool: &PgPool, token: &str, username: &str) -> BotRow {
        let new_bot = NewBot {
            owner_id: Some(1),
            token: token.to_string(),
            username: username.to_string(),
            r#type: BotType::Main,
            is_active: true,
            is_primary: false,
            referral_percentage: Decimal::from_str("0.1").unwrap(),
            created_by: Some(1),
        };
        sqlx::query_as!(
            BotRow,
            r#"
            INSERT INTO bots (owner_id, token, username, type, is_active, is_primary, referral_percentage, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id, owner_id, token, username, type as "type: _", is_active,
                is_primary, referral_percentage, 
                created_at, updated_at, created_by
            "#,
            new_bot.owner_id,
            new_bot.token,
            new_bot.username,
            new_bot.r#type as BotType,
            new_bot.is_active,
            new_bot.is_primary,
            new_bot.referral_percentage,
            new_bot.created_by
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_get_bot(pool: PgPool) {
        let repo = BotRepository::new(Arc::new(pool.clone()));
        let token = "test_token_1";
        let username = "test_bot_1";

        // Create a bot
        let created_bot = create_test_bot(&pool, token, username).await;
        assert_eq!(created_bot.token, token);
        assert_eq!(created_bot.username, username);

        // Get by id
        let fetched_bot_by_id = repo.get_by_id(created_bot.id).await.unwrap();
        assert_eq!(fetched_bot_by_id.id, created_bot.id);

        // Get by token
        let fetched_bot_by_token = repo.get_by_token(token.to_string()).await.unwrap();
        assert_eq!(fetched_bot_by_token.token, token);
    }

    #[sqlx::test]
    async fn test_update_bot(pool: PgPool) {
        let repo = BotRepository::new(Arc::new(pool.clone()));
        let token = "test_token_2";
        let username = "test_bot_2";

        // Create a bot
        let bot = create_test_bot(&pool, token, username).await;

        // Update the bot
        let new_username = "updated_bot_username";
        let update_data = UpdateBot {
            username: Some(new_username.to_string()),
            is_active: Some(false),
            is_primary: Some(true),
            referral_percentage: Some(Decimal::from_str("0.15").unwrap()),
        };
        let updated_bot = repo.update(bot.id, update_data).await.unwrap();
        assert_eq!(updated_bot.username, new_username);
        assert!(!updated_bot.is_active);
        assert!(updated_bot.is_primary);
        assert_eq!(
            updated_bot.referral_percentage,
            Decimal::from_str("0.15").unwrap()
        );

        // Verify the update
        let fetched_bot = repo.get_by_id(bot.id).await.unwrap();
        assert_eq!(fetched_bot.username, new_username);
        assert!(!fetched_bot.is_active);
    }

    #[sqlx::test]
    async fn test_get_list_bots(pool: PgPool) {
        let repo = BotRepository::new(Arc::new(pool.clone()));

        // Create some bots
        create_test_bot(&pool, "list_token_1", "list_bot_1").await;
        create_test_bot(&pool, "list_token_2", "list_bot_2").await;

        // Get the list of bots
        let query = BotListQuery {
            filters: vec![],
            pagination: Pagination {
                page: 1,
                page_size: 10,
            },
            order_by: None,
            order_dir: OrderDir::default(),
            _phantom: std::marker::PhantomData,
        };
        let bots = repo.get_list(query).await.unwrap();
        assert!(!bots.items.is_empty());
        assert!(bots.total >= 2);
    }
}
