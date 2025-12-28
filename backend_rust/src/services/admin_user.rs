use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::ApiResult,
    infrastructure::repositories::admin_user::{AdminUserRepository, AdminUserRepositoryTrait},
    models::admin_user::{AdminUserRow, NewAdminUser, UpdateAdminUser},
    services::topt_encryptor::TotpEncryptor,
};

#[derive(Debug)]
pub struct CreateAdminUser {
    pub login: String,
    pub password: String,
    pub created_by: i64,
}

#[async_trait]
pub trait AdminUserServiceTrait: Send + Sync {
    async fn get_list(&self) -> ApiResult<Vec<AdminUserRow>>;
    async fn create(&self, admin_user: CreateAdminUser) -> ApiResult<AdminUserRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<AdminUserRow>;
    async fn get_by_login(&self, login: &str) -> ApiResult<AdminUserRow>;
    async fn update(&self, id: i64, admin_user: UpdateAdminUser) -> ApiResult<AdminUserRow>;
    async fn delete(&self, id: i64) -> ApiResult<()>;
}

pub struct AdminUserService<R> {
    repo: Arc<R>,
    totp_encryptor: Arc<TotpEncryptor>,
}

impl<R> AdminUserService<R>
where
    R: AdminUserRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>, totp_encryptor: Arc<TotpEncryptor>) -> Self {
        Self {
            repo,
            totp_encryptor,
        }
    }
}

#[async_trait]
impl AdminUserServiceTrait for AdminUserService<AdminUserRepository> {
    async fn get_list(&self) -> ApiResult<Vec<AdminUserRow>> {
        let res = self.repo.get_list().await?;
        Ok(res)
    }

    async fn create(&self, admin_user: CreateAdminUser) -> ApiResult<AdminUserRow> {
        let created = self
            .repo
            .create(NewAdminUser {
                login: admin_user.login,
                created_by: admin_user.created_by,
                hashed_password: bcrypt::hash(&admin_user.password, bcrypt::DEFAULT_COST)?,
                telegram_id: None,
                two_fa_secret: self
                    .totp_encryptor
                    .encrypt(&totp_rs::Secret::generate_secret().to_string())?,
            })
            .await?;
        Ok(created)
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<AdminUserRow> {
        let res = self.repo.get_by_id(id).await?;
        Ok(res)
    }

    async fn get_by_login(&self, login: &str) -> ApiResult<AdminUserRow> {
        let res = self.repo.get_by_login(login).await?;
        Ok(res)
    }

    async fn update(&self, id: i64, admin_user: UpdateAdminUser) -> ApiResult<AdminUserRow> {
        let res = self.repo.update(id, admin_user).await?;
        Ok(res)
    }

    async fn delete(&self, id: i64) -> ApiResult<()> {
        Ok(self.repo.delete(id).await?)
    }
}
