use std::sync::Arc;

use chrono::Duration;
use deadpool_redis::Runtime;
use totp_rs::Algorithm;

#[cfg(feature = "mock-payments-provider")]
use crate::infrastructure::external::payment::mock::MockPaymentsProvider;
#[cfg(feature = "contms-provider")]
use crate::infrastructure::external::products::contms::ContmsProductsProvider;
use crate::{
    config::{self, Config},
    db,
    infrastructure::{
        external::payment::autosales_platform::AutosalesPlatformPaymentsProvider,
        repositories::{
            active_token::ActiveTokenRepository, admin_user::AdminUserRepository,
            analytics::AnalyticsRepository, audit_log::AuditLogRepository, bot::BotRepository,
            broadcast::BroadcastRepository, category::CategoryRepository,
            customer::CustomerRepository, effective_permission::EffectivePermissionRepository,
            image::ImageRepository, order::OrderRepository, order_item::OrderItemRepository,
            payment_invoice::PaymentInvoiceRepository, permission::PermissionRepository,
            products::ProductRepository, role::RoleRepository,
            role_permission::RolePermissionRepository, settings::SettingsRepository,
            stock_movement::StockMovementRepository, temporary_token::TemporaryTokenRepository,
            transaction::TransactionRepository, user_permission::UserPermissionRepository,
            user_role::UserRoleRepository, user_subscription::UserSubscriptionRepository,
        },
    },
    services::{
        admin_user::AdminUserService,
        analytics::AnalyticsService,
        audit_log::AuditLogService,
        auth::{AuthService, AuthServiceConfig},
        bot::BotService,
        broadcast::BroadcastService,
        captcha::CaptchaService,
        category::CategoryService,
        customer::CustomerService,
        image::ImageService,
        notification_service::NotificationService,
        order::OrderService,
        order_item::OrderItemService,
        payment_invoice::PaymentInvoiceService,
        payment_processing_service::PaymentProcessingService,
        permission::PermissionService,
        product::ProductService,
        purchase::PurchaseService,
        role::RoleService,
        role_permission::RolePermissionService,
        settings::SettingsService,
        stock_movement::StockMovementService,
        topt_encryptor::TotpEncryptor,
        transaction::TransactionService,
        user_subscription::UserSubscriptionService,
    },
};

type AuditLogShortType = AuditLogService<AuditLogRepository>;

type CategoryServiceShortType = CategoryService<CategoryRepository, AuditLogShortType>;

type ProductServiceShortType = ProductService<
    ProductRepository,
    StockMovementRepository,
    AuditLogShortType,
    SettingsRepository,
    CategoryServiceShortType,
>;

type TransactionServiceShortType = TransactionService<TransactionRepository>;

type CustomerServiceShortType = CustomerService<CustomerRepository, AuditLogShortType>;

type PaymentInvoiceShortType = PaymentInvoiceService<
    PaymentInvoiceRepository,
    AuditLogShortType,
    MockPaymentsProvider,
    SettingsRepository,
    AutosalesPlatformPaymentsProvider,
>;

type OrderItemServiceShortType = OrderItemService<OrderItemRepository, StockMovementRepository>;

type BotServiceShortType =
    BotService<BotRepository, SettingsRepository, AuditLogShortType, TransactionRepository>;

type PurchaseServiceShortType = PurchaseService<
    TransactionServiceShortType,
    CustomerServiceShortType,
    OrderItemServiceShortType,
    OrderService<OrderRepository, OrderItemRepository>,
    ProductServiceShortType,
    ContmsProductsProvider,
    UserSubscriptionService<UserSubscriptionRepository>,
    BotServiceShortType,
>;

#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
    pub redis_pool: Arc<deadpool_redis::Pool>,
    pub config: config::Config,
    pub auth_service: Arc<
        AuthService<
            ActiveTokenRepository,
            TemporaryTokenRepository,
            AdminUserRepository,
            EffectivePermissionRepository,
        >,
    >,
    pub category_service: Arc<CategoryServiceShortType>,
    pub admin_user_service:
        Arc<AdminUserService<AdminUserRepository, UserRoleRepository, AuditLogShortType>>,
    pub role_service: Arc<RoleService<RoleRepository>>,
    pub permission_service: Arc<PermissionService<PermissionRepository, UserPermissionRepository>>,
    pub role_permission_service: Arc<RolePermissionService<RolePermissionRepository>>,
    pub transaction_service: Arc<TransactionService<TransactionRepository>>,
    pub product_service: Arc<ProductServiceShortType>,
    pub image_service: Arc<ImageService<ImageRepository>>,
    pub stock_movement_service: Arc<StockMovementService<StockMovementRepository>>,
    pub customer_service: Arc<CustomerServiceShortType>,
    pub settings_service: Arc<SettingsService<SettingsRepository, AuditLogShortType>>,
    pub audit_logs_service: Arc<AuditLogShortType>,
    pub bot_service: Arc<BotServiceShortType>,
    pub order_service: Arc<OrderService<OrderRepository, OrderItemRepository>>,
    pub captcha_service: Arc<CaptchaService>,
    pub payment_invoice_service: Arc<PaymentInvoiceShortType>,
    pub notification_service: Arc<NotificationService>,
    pub broadcast_service: Arc<BroadcastService<BroadcastRepository, AuditLogShortType>>,
    pub payment_processing_service: Arc<
        PaymentProcessingService<
            TransactionServiceShortType,
            PaymentInvoiceShortType,
            NotificationService,
            CustomerServiceShortType,
        >,
    >,
    pub order_item_service: Arc<OrderItemServiceShortType>,
    pub user_subscription_service: Arc<UserSubscriptionService<UserSubscriptionRepository>>,
    pub purchase_service: Arc<PurchaseServiceShortType>,
    pub client: Arc<reqwest::Client>,
    #[cfg(feature = "contms-provider")]
    pub contms_products_provider: Arc<ContmsProductsProvider>,
    #[cfg(feature = "mock-payments-provider")]
    pub mock_payments_provider: Arc<MockPaymentsProvider>,
    pub platform_payments_provider: Arc<AutosalesPlatformPaymentsProvider>,
    pub analytics_service: Arc<AnalyticsService<AnalyticsRepository>>,
}

impl AppState {
    pub fn new(db: db::Database, config: Config) -> Self {
        let db_pool = Arc::new(db.get_pool().clone());
        let client = Arc::new(reqwest::Client::new());
        let audit_log_repo = Arc::new(AuditLogRepository::new(db_pool.clone()));
        let audit_logs_service = Arc::new(AuditLogService::new(audit_log_repo));
        let active_token_repo = Arc::new(ActiveTokenRepository::new(db_pool.clone()));
        let temp_token_repo = Arc::new(TemporaryTokenRepository::new(db_pool.clone()));
        let admin_user_repo = Arc::new(AdminUserRepository::new(db_pool.clone()));
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
        let category_service = Arc::new(CategoryService::new(
            category_repo,
            audit_logs_service.clone(),
        ));
        let user_role_repo = Arc::new(UserRoleRepository::new(db_pool.clone()));
        let admin_user_service = Arc::new(AdminUserService::new(
            db_pool.clone(),
            admin_user_repo,
            user_role_repo,
            totp_encryptor,
            audit_logs_service.clone(),
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
        let transaction_service = Arc::new(TransactionService::new(transaction_repo.clone()));
        let product_repo = Arc::new(ProductRepository::new(db_pool.clone()));
        let settings_repo = Arc::new(SettingsRepository::new(db_pool.clone()));
        let settings_service = Arc::new(SettingsService::new(
            settings_repo.clone(),
            audit_logs_service.clone(),
        ));
        let stock_movement_repo = Arc::new(StockMovementRepository::new(db_pool.clone()));
        let product_service = Arc::new(ProductService::new(
            product_repo,
            stock_movement_repo.clone(),
            settings_repo.clone(),
            audit_logs_service.clone(),
            category_service.clone(),
        ));
        let image_repo = Arc::new(ImageRepository::new(db_pool.clone()));
        let image_service = Arc::new(ImageService::new(
            image_repo,
            config.image_upload_path.clone(),
        ));
        let stock_movement_service =
            Arc::new(StockMovementService::new(stock_movement_repo.clone()));
        let customer_repo = Arc::new(CustomerRepository::new(db_pool.clone()));
        let customer_service = Arc::new(CustomerService::new(
            customer_repo,
            audit_logs_service.clone(),
        ));

        let bot_service = Arc::new(BotService::new(
            Arc::new(BotRepository::new(db_pool.clone())),
            settings_repo.clone(),
            transaction_repo.clone(),
            audit_logs_service.clone(),
            client.clone(),
        ));
        let order_item_repo = Arc::new(OrderItemRepository::new(db_pool.clone()));
        let order_service = Arc::new(OrderService::new(
            Arc::new(OrderRepository::new(db_pool.clone())),
            order_item_repo.clone(),
        ));
        let captcha_service = Arc::new(CaptchaService::new(
            client.clone(),
            config.captcha_api_url.clone(),
        ));
        let order_item_service = Arc::new(OrderItemService::new(
            order_item_repo.clone(),
            stock_movement_repo.clone(),
        ));
        #[cfg(feature = "mock-payments-provider")]
        let mock_payments_provider = Arc::new(MockPaymentsProvider::new(
            client.clone(),
            config.mock_payments_provider_url.clone(),
        ));
        let platform_payments_provider = Arc::new(AutosalesPlatformPaymentsProvider::new(
            client.clone(),
            config.platform_payment_system_base_url.clone(),
            config.platform_payment_system_login.clone(),
            config.platform_payment_system_password.clone(),
            config.platform_payment_system_2fa_key.clone(),
        ));
        let payment_invoice_service = Arc::new(PaymentInvoiceService::new(
            Arc::new(PaymentInvoiceRepository::new(db_pool.clone())),
            settings_repo.clone(),
            mock_payments_provider.clone(),
            audit_logs_service.clone(),
            platform_payments_provider.clone(),
        ));
        #[cfg(feature = "contms-provider")]
        let contms_products_provider = Arc::new(ContmsProductsProvider::new(
            client.clone(),
            config.contms_api_url.clone(),
        ));

        let redis_config = deadpool_redis::Config::from_url(format!(
            "redis://{}:{}",
            config.redis_host, config.redis_port
        ));
        let redis_pool = Arc::new(
            redis_config
                .create_pool(Some(Runtime::Tokio1))
                .expect("Failed to create redis pool"),
        );

        let notification_service = Arc::new(NotificationService::new(redis_pool.clone()));
        let payment_processing_service = Arc::new(PaymentProcessingService::new(
            transaction_service.clone(),
            payment_invoice_service.clone(),
            notification_service.clone(),
            customer_service.clone(),
        ));
        let user_subscription_service = Arc::new(UserSubscriptionService::new(Arc::new(
            UserSubscriptionRepository::new(db_pool.clone()),
        )));
        let purchase_service = Arc::new(PurchaseService::new(
            transaction_service.clone(),
            customer_service.clone(),
            product_service.clone(),
            order_service.clone(),
            order_item_service.clone(),
            contms_products_provider.clone(),
            user_subscription_service.clone(),
            bot_service.clone(),
        ));
        let broadcast_service = Arc::new(BroadcastService::new(
            Arc::new(BroadcastRepository::new(db_pool.clone())),
            audit_logs_service.clone(),
        ));
        let analytics_service = Arc::new(AnalyticsService::new(Arc::new(
            AnalyticsRepository::new(db_pool.clone()),
        )));

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
            customer_service,
            settings_service,
            audit_logs_service,
            client,
            bot_service,
            order_service,
            captcha_service,
            payment_invoice_service,
            notification_service,
            payment_processing_service,
            redis_pool,
            order_item_service,
            purchase_service,
            user_subscription_service,
            broadcast_service,
            #[cfg(feature = "contms-provider")]
            contms_products_provider,
            #[cfg(feature = "mock-payments-provider")]
            mock_payments_provider,
            platform_payments_provider,
            analytics_service,
        }
    }
}
