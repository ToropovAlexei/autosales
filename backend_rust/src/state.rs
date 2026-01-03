use std::sync::Arc;

use chrono::Duration;
use totp_rs::Algorithm;

use crate::{
    config::{self, Config},
    db,
    infrastructure::repositories::{
        active_token::ActiveTokenRepository, admin_user::AdminUserRepository,
        admin_user_with_roles::AdminUserWithRolesRepository, category::CategoryRepository,
        effective_permission::EffectivePermissionRepository, image::ImageRepository,
        permission::PermissionRepository, products::ProductRepository, role::RoleRepository,
        role_permission::RolePermissionRepository, stock_movement::StockMovementRepository,
        temporary_token::TemporaryTokenRepository, transaction::TransactionRepository,
        user_permission::UserPermissionRepository, user_role::UserRoleRepository,
    },
    services::{
        admin_user::AdminUserService,
        auth::{AuthService, AuthServiceConfig},
        category::CategoryService,
        image::ImageService,
        permission::PermissionService,
        product::ProductService,
        role::RoleService,
        role_permission::RolePermissionService,
        stock_movement::StockMovementService,
        topt_encryptor::TotpEncryptor,
        transaction::TransactionService,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
    pub config: config::Config,
    pub auth_service: Arc<
        AuthService<
            ActiveTokenRepository,
            TemporaryTokenRepository,
            AdminUserRepository,
            EffectivePermissionRepository,
        >,
    >,
    pub category_service: Arc<CategoryService<CategoryRepository>>,
    pub admin_user_service: Arc<
        AdminUserService<AdminUserRepository, AdminUserWithRolesRepository, UserRoleRepository>,
    >,
    pub role_service: Arc<RoleService<RoleRepository>>,
    pub permission_service: Arc<PermissionService<PermissionRepository, UserPermissionRepository>>,
    pub role_permission_service: Arc<RolePermissionService<RolePermissionRepository>>,
    pub transaction_service: Arc<TransactionService<TransactionRepository>>,
    pub product_service: Arc<ProductService<ProductRepository, StockMovementRepository>>,
    pub image_service: Arc<ImageService<ImageRepository>>,
    pub stock_movement_service: Arc<StockMovementService<StockMovementRepository>>,
}

impl AppState {
    pub fn new(db: db::Database, config: Config) -> Self {
        let db_pool = Arc::new(db.get_pool().clone());
        let active_token_repo = Arc::new(ActiveTokenRepository::new(db_pool.clone()));
        let temp_token_repo = Arc::new(TemporaryTokenRepository::new(db_pool.clone()));
        let admin_user_repo = Arc::new(AdminUserRepository::new(db_pool.clone()));
        let admin_user_with_roles_repo =
            Arc::new(AdminUserWithRolesRepository::new(db_pool.clone()));
        let effective_permission_repo =
            Arc::new(EffectivePermissionRepository::new(db_pool.clone()));
        let totp_encryptor = Arc::new(
            TotpEncryptor::new(&config.totp_encode_secret.clone())
                .expect("Failed to init totp_encryptor"),
        );
        let auth_service = Arc::new(AuthService::new(
            active_token_repo,
            temp_token_repo,
            admin_user_repo.clone(),
            effective_permission_repo,
            totp_encryptor.clone(),
            AuthServiceConfig {
                jwt_secret: config.jwt_secret.clone(),
                totp_encode_secret: config.totp_encode_secret.clone(),
                two_fa_token_ttl: Duration::minutes(config.two_fa_token_ttl_minutes),
                totp_algorithm: Algorithm::SHA1,
                totp_digits: 6,
                totp_skew: 1,
                totp_step: 30,
                access_token_ttl: Duration::minutes(config.access_token_ttl_minutes),
                refresh_token_ttl: Duration::minutes(config.refresh_token_ttl_minutes),
            },
        ));
        let category_repo = Arc::new(CategoryRepository::new(db_pool.clone()));
        let category_service = Arc::new(CategoryService::new(category_repo));
        let user_role_repo = Arc::new(UserRoleRepository::new(db_pool.clone()));
        let admin_user_service = Arc::new(AdminUserService::new(
            admin_user_repo,
            admin_user_with_roles_repo,
            user_role_repo,
            totp_encryptor,
        ));
        let role_repo = Arc::new(RoleRepository::new(db_pool.clone()));
        let role_service = Arc::new(RoleService::new(role_repo));
        let permission_repo = Arc::new(PermissionRepository::new(db_pool.clone()));
        let user_permission_repo = Arc::new(UserPermissionRepository::new(db_pool.clone()));
        let permission_service = Arc::new(PermissionService::new(
            permission_repo,
            user_permission_repo,
        ));
        let role_permission_repo = Arc::new(RolePermissionRepository::new(db_pool.clone()));
        let role_permission_service = Arc::new(RolePermissionService::new(role_permission_repo));
        let transaction_repo = Arc::new(TransactionRepository::new(db_pool.clone()));
        let transaction_service = Arc::new(TransactionService::new(transaction_repo));
        let product_repo = Arc::new(ProductRepository::new(db_pool.clone()));
        let stock_movement_repo = Arc::new(StockMovementRepository::new(db_pool.clone()));
        let product_service = Arc::new(ProductService::new(
            product_repo,
            stock_movement_repo.clone(),
        ));
        let image_repo = Arc::new(ImageRepository::new(db_pool.clone()));
        let image_service = Arc::new(ImageService::new(
            image_repo,
            config.image_upload_path.clone(),
        ));
        let stock_movement_service = Arc::new(StockMovementService::new(stock_movement_repo));

        Self {
            db,
            config,
            auth_service,
            category_service,
            admin_user_service,
            role_service,
            permission_service,
            role_permission_service,
            transaction_service,
            product_service,
            image_service,
            stock_movement_service,
        }
    }
}
