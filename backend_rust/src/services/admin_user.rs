use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    errors::api::ApiResult,
    infrastructure::repositories::{
        admin_user::{AdminUserRepository, AdminUserRepositoryTrait},
        admin_user_with_roles::{AdminUserWithRolesRepository, AdminUserWithRolesRepositoryTrait},
        audit_log::AuditLogRepository,
        user_role::{UserRoleRepository, UserRoleRepositoryTrait},
    },
    middlewares::context::RequestContext,
    models::{
        admin_user::{AdminUserRow, NewAdminUser, UpdateAdminUser},
        admin_user_with_roles::AdminUserWithRolesRow,
        audit_log::{AuditAction, AuditStatus, NewAuditLog},
        user_role::AssignUserRoles,
    },
    services::{
        audit_log::{AuditLogService, AuditLogServiceTrait},
        topt_encryptor::TotpEncryptor,
    },
};

#[derive(Debug)]
pub struct CreateAdminUser {
    pub login: String,
    pub password: String,
    pub created_by: i64,
    pub roles: Vec<i64>,
}

#[derive(Debug)]
pub struct CreatedAdminUser {
    pub id: i64,
    pub login: String,
    pub hashed_password: String,
    pub two_fa_secret: String,
    pub two_fa_qr_code: String,
    pub telegram_id: Option<i64>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct DeleteAdminUserCommand {
    pub id: i64,
    pub deleted_by: i64,
}

#[derive(Debug)]
pub struct UpdateAdminUserCommand {
    pub login: Option<String>,
    pub password: Option<String>,
    pub telegram_id: Option<i64>,
    pub roles: Option<Vec<i64>>,
    pub updated_by: i64,
}

#[async_trait]
pub trait AdminUserServiceTrait: Send + Sync {
    async fn get_list(&self) -> ApiResult<Vec<AdminUserRow>>;
    async fn get_all_users_with_roles(&self) -> ApiResult<Vec<AdminUserWithRolesRow>>;
    async fn create(
        &self,
        admin_user: CreateAdminUser,
        ctx: RequestContext,
    ) -> ApiResult<CreatedAdminUser>;
    async fn get_by_id(&self, id: i64) -> ApiResult<AdminUserRow>;
    async fn get_by_login(&self, login: &str) -> ApiResult<AdminUserRow>;
    async fn update(
        &self,
        id: i64,
        admin_user: UpdateAdminUserCommand,
        ctx: RequestContext,
    ) -> ApiResult<AdminUserRow>;
    async fn delete(&self, command: DeleteAdminUserCommand, ctx: RequestContext) -> ApiResult<()>;
}

pub struct AdminUserService<R, T, S, A> {
    repo: Arc<R>,
    admin_user_with_roles_repo: Arc<T>,
    user_role_repo: Arc<S>,
    totp_encryptor: Arc<TotpEncryptor>,
    audit_log_service: Arc<A>,
}

impl<R, T, S, A> AdminUserService<R, T, S, A>
where
    R: AdminUserRepositoryTrait + Send + Sync,
    T: AdminUserWithRolesRepositoryTrait + Send + Sync,
    S: UserRoleRepositoryTrait + Send + Sync,
    A: AuditLogServiceTrait + Send + Sync,
{
    pub fn new(
        repo: Arc<R>,
        admin_user_with_roles_repo: Arc<T>,
        user_role_repo: Arc<S>,
        totp_encryptor: Arc<TotpEncryptor>,
        audit_log_service: Arc<A>,
    ) -> Self {
        Self {
            repo,
            admin_user_with_roles_repo,
            totp_encryptor,
            user_role_repo,
            audit_log_service,
        }
    }
}

#[async_trait]
impl AdminUserServiceTrait
    for AdminUserService<
        AdminUserRepository,
        AdminUserWithRolesRepository,
        UserRoleRepository,
        AuditLogService<AuditLogRepository>,
    >
{
    async fn get_list(&self) -> ApiResult<Vec<AdminUserRow>> {
        let res = self.repo.get_list().await?;
        Ok(res)
    }

    async fn get_all_users_with_roles(&self) -> ApiResult<Vec<AdminUserWithRolesRow>> {
        let res = self.admin_user_with_roles_repo.get_list().await?;
        Ok(res)
    }

    async fn create(
        &self,
        admin_user: CreateAdminUser,
        ctx: RequestContext,
    ) -> ApiResult<CreatedAdminUser> {
        let secret = totp_rs::Secret::generate_secret().to_encoded().to_string();
        let created = self
            .repo
            .create(NewAdminUser {
                login: admin_user.login,
                created_by: admin_user.created_by,
                hashed_password: bcrypt::hash(&admin_user.password, bcrypt::DEFAULT_COST)?,
                telegram_id: None,
                two_fa_secret: self.totp_encryptor.encrypt(&secret)?,
            })
            .await?;
        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::UserCreate,
                status: AuditStatus::Success,
                admin_user_id: Some(admin_user.created_by),
                customer_id: None,
                error_message: None,
                ip_address: ctx.ip_address,
                new_values: serde_json::to_value(created.clone()).ok(),
                old_values: None,
                request_id: Some(ctx.request_id),
                target_id: created.id.to_string(),
                target_table: "admin_users".to_string(),
                user_agent: ctx.user_agent.clone(),
            })
            .await?;
        self.user_role_repo
            .assign_roles_to_admin_user(AssignUserRoles {
                created_by: admin_user.created_by,
                user_id: created.id,
                roles: admin_user.roles.clone(),
            })
            .await?;
        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::RoleGrant,
                status: AuditStatus::Success,
                admin_user_id: Some(admin_user.created_by),
                customer_id: None,
                error_message: None,
                ip_address: ctx.ip_address,
                new_values: serde_json::to_value(admin_user.roles).ok(),
                old_values: None,
                request_id: Some(ctx.request_id),
                target_id: created.id.to_string(),
                target_table: "user_roles".to_string(),
                user_agent: ctx.user_agent,
            })
            .await?;
        Ok(CreatedAdminUser {
            created_at: created.created_at,
            created_by: created.created_by,
            id: created.id,
            is_system: created.is_system,
            two_fa_qr_code: self
                .totp_encryptor
                .generate_qr_code(&created.login, &secret)?,
            login: created.login,
            telegram_id: created.telegram_id,
            updated_at: created.updated_at,
            two_fa_secret: secret.to_string(),
            hashed_password: created.hashed_password,
            deleted_at: created.deleted_at,
        })
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<AdminUserRow> {
        let res = self.repo.get_by_id(id).await?;
        Ok(res)
    }

    async fn get_by_login(&self, login: &str) -> ApiResult<AdminUserRow> {
        let res = self.repo.get_by_login(login).await?;
        Ok(res)
    }

    async fn update(
        &self,
        id: i64,
        admin_user: UpdateAdminUserCommand,
        ctx: RequestContext,
    ) -> ApiResult<AdminUserRow> {
        let old_values = self.repo.get_by_id(id).await?;
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
        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::UserUpdate,
                status: AuditStatus::Success,
                admin_user_id: Some(admin_user.updated_by),
                customer_id: None,
                error_message: None,
                ip_address: ctx.ip_address,
                new_values: serde_json::to_value(res.clone()).ok(),
                old_values: serde_json::to_value(old_values).ok(),
                request_id: Some(ctx.request_id),
                target_id: res.id.to_string(),
                target_table: "admin_users".to_string(),
                user_agent: ctx.user_agent,
            })
            .await?;
        if let Some(roles) = admin_user.roles {
            self.user_role_repo
                .assign_roles_to_admin_user(AssignUserRoles {
                    created_by: res.created_by,
                    user_id: res.id,
                    roles,
                })
                .await?;
        }
        Ok(res)
    }

    async fn delete(&self, command: DeleteAdminUserCommand, ctx: RequestContext) -> ApiResult<()> {
        let admin_user = self.repo.get_by_id(command.id).await?;
        self.repo.delete(command.id).await?;
        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::UserDelete,
                status: AuditStatus::Success,
                admin_user_id: Some(command.deleted_by),
                customer_id: None,
                error_message: None,
                ip_address: ctx.ip_address,
                new_values: None,
                old_values: serde_json::to_value(admin_user).ok(),
                request_id: Some(ctx.request_id),
                target_id: command.id.to_string(),
                target_table: "admin_users".to_string(),
                user_agent: ctx.user_agent,
            })
            .await?;
        Ok(())
    }
}
