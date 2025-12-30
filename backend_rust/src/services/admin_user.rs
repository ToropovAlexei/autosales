use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::ApiResult,
    infrastructure::repositories::{
        admin_user::{AdminUserRepository, AdminUserRepositoryTrait},
        admin_user_with_roles::{AdminUserWithRolesRepository, AdminUserWithRolesRepositoryTrait},
    },
    models::{
        admin_user::{AdminUserRow, NewAdminUser, UpdateAdminUser},
        admin_user_with_roles::AdminUserWithRolesRow,
    },
    services::topt_encryptor::TotpEncryptor,
};

#[derive(Debug)]
pub struct CreateAdminUser {
    pub login: String,
    pub password: String,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct UpdateAdminUserCommand {
    pub login: Option<String>,
    pub password: Option<String>,
    pub telegram_id: Option<i64>,
}

#[async_trait]
pub trait AdminUserServiceTrait: Send + Sync {
    async fn get_list(&self) -> ApiResult<Vec<AdminUserRow>>;
    async fn get_all_users_with_roles(&self) -> ApiResult<Vec<AdminUserWithRolesRow>>;
    async fn create(&self, admin_user: CreateAdminUser) -> ApiResult<AdminUserRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<AdminUserRow>;
    async fn get_by_login(&self, login: &str) -> ApiResult<AdminUserRow>;
    async fn update(&self, id: i64, admin_user: UpdateAdminUserCommand) -> ApiResult<AdminUserRow>;
    async fn delete(&self, id: i64) -> ApiResult<()>;
}

pub struct AdminUserService<R, T> {
    repo: Arc<R>,
    admin_user_with_roles_repo: Arc<T>,
    totp_encryptor: Arc<TotpEncryptor>,
}

impl<R, T> AdminUserService<R, T>
where
    R: AdminUserRepositoryTrait + Send + Sync,
    T: AdminUserWithRolesRepositoryTrait + Send + Sync,
{
    pub fn new(
        repo: Arc<R>,
        admin_user_with_roles_repo: Arc<T>,
        totp_encryptor: Arc<TotpEncryptor>,
    ) -> Self {
        Self {
            repo,
            admin_user_with_roles_repo,
            totp_encryptor,
        }
    }
}

#[async_trait]
impl AdminUserServiceTrait for AdminUserService<AdminUserRepository, AdminUserWithRolesRepository> {
    async fn get_list(&self) -> ApiResult<Vec<AdminUserRow>> {
        let res = self.repo.get_list().await?;
        Ok(res)
    }

    async fn get_all_users_with_roles(&self) -> ApiResult<Vec<AdminUserWithRolesRow>> {
        let res = self.admin_user_with_roles_repo.get_list().await?;
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

    async fn update(&self, id: i64, admin_user: UpdateAdminUserCommand) -> ApiResult<AdminUserRow> {
        let res = self
            .repo
            .update(
                id,
                UpdateAdminUser {
                    hashed_password: admin_user
                        .password
                        .map(|p| bcrypt::hash(&p, bcrypt::DEFAULT_COST))
                        .transpose()?,
                    login: admin_user.login,
                    telegram_id: admin_user.telegram_id,
                    two_fa_secret: None,
                },
            )
            .await?;
        Ok(res)
    }

    async fn delete(&self, id: i64) -> ApiResult<()> {
        Ok(self.repo.delete(id).await?)
    }
}
