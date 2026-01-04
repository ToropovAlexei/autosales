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
}
