use shared_dtos::{
    admin_user::{
        AdminUserWithRolesAdminResponse, NewAdminUserAdminRequest, UpdateAdminUserAdminRequest,
    },
    analytics::BotAnalyticsBotResponse,
    audit_log::AuditLogAdminResponse,
    auth::{
        LoginStep1AdminRequest, LoginStep1AdminResponse, LoginStep2AdminRequest,
        LoginStep2AdminResponse,
    },
    bot::{
        BotAdminResponse, BotBotResponse, NewBotAdminRequest, NewBotBotRequest,
        UpdateBotAdminRequest, UpdateBotBotRequest,
    },
    can_operate::CanOperateBotResponse,
    captcha::CaptchaBotResponse,
    category::{CategoryAdminResponse, NewCategoryAdminRequest, UpdateCategoryAdminRequest},
    customer::{CustomerBotResponse, NewCustomerBotRequest, UpdateCustomerBotRequest},
    error::ApiErrorResponse,
    image::ImageAdminResponse,
    invoice::{
        GatewayBotResponse, NewPaymentInvoiceBotRequest, PaymentInvoiceBotResponse,
        UpdatePaymentInvoiceBotRequest,
    },
    list_response::ListResponse,
    order::{EnrichedOrderBotResponse, OrderItemBotResponse, PurchaseBotResponse},
    permission::PermissionAdminResponse,
    product::ProductType,
    role::{NewRoleAdminRequest, RoleAdminResponse, UpdateRoleAdminRequest},
    settings::SettingsBotResponse,
    transaction::TransactionAdminResponse,
};
use std::sync::Arc;
use tokio::signal;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

#[cfg(feature = "contms-provider")]
use backend_rust::workers::contms_products_sync::contms_products_sync_task;
use backend_rust::{
    config::Config,
    create_app,
    db::Database,
    init_tracing,
    presentation::{
        admin::{
            dtos::{
                broadcast::{BroadcastResponse, NewBroadcastRequest},
                customer::CustomerAdminResponse,
                dashboard::{
                    CategorySalesResponse, DashboardOverviewResponse, SalesOverTimeResponse,
                    StatWithTrendResponse, TimeSeriesDashboardDataResponse,
                    TimeSeriesPointResponse, TopProductResponse,
                },
                order::OrderAdminResponse,
                payment_invoice::PaymentInvoiceAdminResponse,
                product::{
                    NewProductAdminRequest, ProductAdminResponse, UpdateProductAdminRequest,
                },
                settings::{
                    BotSettingsAdminResponse, PricingSettingsAdminResponse,
                    UpdateBotSettingsAdminRequest, UpdatePricingSettingsAdminRequest,
                },
                stock_movement::StockMovementAdminResponse,
                store_balance::StoreBalanceAdminResponse,
            },
            handlers as admin_handlers,
        },
        bot::handlers as bot_handlers,
        images::handlers as images_handlers,
        webhook::handlers as webhook_handlers,
    },
    run_migrations,
    state::AppState,
    workers::{broadcasts::broadcasts_task, pending_payments::pending_payments_task},
};

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    security(
        ("admin_bearer" = []),
        ("service_api_key" = [], "bot_id" = [])
    ),
    paths(
        admin_handlers::auth::login_step1,
        admin_handlers::auth::login_step2,
        admin_handlers::auth::logout,
        admin_handlers::category::create_category,
        admin_handlers::category::delete_category,
        admin_handlers::category::get_category,
        admin_handlers::category::list_categories,
        admin_handlers::category::update_category,
        admin_handlers::product::create_product,
        admin_handlers::product::delete_product,
        admin_handlers::product::get_product,
        admin_handlers::product::list_products,
        admin_handlers::product::update_product,
        admin_handlers::product::upload_products,
        admin_handlers::image::create_image,
        admin_handlers::image::delete_image,
        admin_handlers::image::list_images,
        admin_handlers::customer::list_customers,
        admin_handlers::customer::get_customer,
        admin_handlers::customer::update_customer,
        admin_handlers::admin_user::list_admin_users,
        admin_handlers::admin_user::get_admin_user,
        admin_handlers::admin_user::create_admin_user,
        admin_handlers::admin_user::update_admin_user,
        admin_handlers::admin_user::delete_admin_user,
        admin_handlers::admin_user::get_admin_user_permissions,
        admin_handlers::admin_user::update_admin_user_permissions,
        admin_handlers::role::list_roles,
        admin_handlers::role::create_role,
        admin_handlers::role::update_role,
        admin_handlers::role::delete_role,
        admin_handlers::role::get_role_permissions,
        admin_handlers::role::update_role_permissions,
        admin_handlers::permission::list_permissions,
        admin_handlers::settings::get_bot_settings,
        admin_handlers::settings::get_pricing_settings,
        admin_handlers::settings::update_bot_settings,
        admin_handlers::settings::update_pricing_settings,
        admin_handlers::me::get_me,
        admin_handlers::me::get_me_permissions,
        admin_handlers::transaction::list_transactions,
        admin_handlers::audit_log::list_audit_logs,
        admin_handlers::stock_movement::list_stock_movement,
        admin_handlers::bot::create_bot,
        admin_handlers::bot::list_bots,
        admin_handlers::bot::update_bot,
        admin_handlers::order::list_orders,
        admin_handlers::store_balance::get_store_balance,
        admin_handlers::broadcast::create_broadcast,
        admin_handlers::dashboard::get_dashboard_stats,
        admin_handlers::dashboard::get_time_series,
        admin_handlers::dashboard::get_top_products,
        admin_handlers::dashboard::get_sales_by_category,
        admin_handlers::payment_invoice::list_payment_invoices,
        admin_handlers::dev::reset_test_data,
        bot_handlers::bot::create_bot,
        bot_handlers::bot::get_bot,
        bot_handlers::bot::list_bots,
        bot_handlers::bot::update_bot,
        bot_handlers::bot::delete_bot,
        bot_handlers::bot::get_primary_bots,
        bot_handlers::can_operate::can_operate,
        bot_handlers::captcha::get_captcha,
        bot_handlers::category::list_categories,
        bot_handlers::customer::get_customer,
        bot_handlers::customer::create_customer,
        bot_handlers::customer::update_customer,
        bot_handlers::customer::get_customer_invoices,
        bot_handlers::customer::get_customer_orders,
        bot_handlers::customer::get_customer_subscriptions,
        bot_handlers::customer::get_customer_referral_analytics,
        bot_handlers::customer::update_customer_last_seen,
        bot_handlers::gateway::get_gateways,
        bot_handlers::invoice::list_invoices,
        bot_handlers::invoice::get_invoice,
        bot_handlers::invoice::create_invoice,
        bot_handlers::invoice::update_invoice,
        bot_handlers::invoice::confirm_invoice,
        bot_handlers::invoice::cancel_invoice,
        bot_handlers::invoice::send_invoice_receipt,
        bot_handlers::order::purchase,
        bot_handlers::order::get_order,
        bot_handlers::product::list_products,
        bot_handlers::product::get_product,
        bot_handlers::settings::get_settings,
        images_handlers::image::get_image,
        #[cfg(feature = "mock-payments-provider")]
        webhook_handlers::payment::mock_payments_provider_webhook,
    ),
    components(schemas(
        CategoryAdminResponse,
        NewCategoryAdminRequest,
        UpdateCategoryAdminRequest,
        ProductAdminResponse,
        NewProductAdminRequest,
        UpdateProductAdminRequest,
        ProductType,
        ImageAdminResponse,
        ListResponse<CategoryAdminResponse>,
        ListResponse<ProductAdminResponse>,
        ListResponse<CustomerAdminResponse>,
        ListResponse<BotAdminResponse>,
        ListResponse<AdminUserWithRolesAdminResponse>,
        ListResponse<RoleAdminResponse>,
        ListResponse<PermissionAdminResponse>,
        ListResponse<OrderAdminResponse>,
        ListResponse<TransactionAdminResponse>,
        ListResponse<StockMovementAdminResponse>,
        ListResponse<AuditLogAdminResponse>,
        ListResponse<BroadcastResponse>,
        ListResponse<CustomerBotResponse>,
        ListResponse<BotBotResponse>,
        ListResponse<GatewayBotResponse>,
        ListResponse<PaymentInvoiceBotResponse>,
        ListResponse<EnrichedOrderBotResponse>,
        ListResponse<PaymentInvoiceAdminResponse>,
        ListResponse<BotAnalyticsBotResponse>,
        CustomerAdminResponse,
        AdminUserWithRolesAdminResponse,
        BotAdminResponse,
        NewBotAdminRequest,
        UpdateBotAdminRequest,
        NewAdminUserAdminRequest,
        UpdateAdminUserAdminRequest,
        NewRoleAdminRequest,
        UpdateRoleAdminRequest,
        RoleAdminResponse,
        PermissionAdminResponse,
        OrderAdminResponse,
        TransactionAdminResponse,
        AuditLogAdminResponse,
        StoreBalanceAdminResponse,
        BroadcastResponse,
        NewBroadcastRequest,
        DashboardOverviewResponse,
        StatWithTrendResponse,
        TimeSeriesPointResponse,
        SalesOverTimeResponse,
        TimeSeriesDashboardDataResponse,
        TopProductResponse,
        CategorySalesResponse,
        PricingSettingsAdminResponse,
        BotSettingsAdminResponse,
        UpdatePricingSettingsAdminRequest,
        UpdateBotSettingsAdminRequest,
        LoginStep1AdminRequest,
        LoginStep1AdminResponse,
        LoginStep2AdminRequest,
        LoginStep2AdminResponse,
        StockMovementAdminResponse,
        BotBotResponse,
        NewBotBotRequest,
        UpdateBotBotRequest,
        CustomerBotResponse,
        NewCustomerBotRequest,
        UpdateCustomerBotRequest,
        CanOperateBotResponse,
        CaptchaBotResponse,
        GatewayBotResponse,
        PaymentInvoiceBotResponse,
        NewPaymentInvoiceBotRequest,
        UpdatePaymentInvoiceBotRequest,
        PaymentInvoiceAdminResponse,
        BotAnalyticsBotResponse,
        EnrichedOrderBotResponse,
        OrderItemBotResponse,
        PurchaseBotResponse,
        SettingsBotResponse,
        ApiErrorResponse,
        admin_handlers::dev::ResetTestDataResponse,
    ))
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "admin_bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("UUID")
                    .build(),
            ),
        );
        components.add_security_scheme(
            "service_api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-KEY"))),
        );
        components.add_security_scheme(
            "bot_id",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-BOT-ID"))),
        );
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let config = Config::from_env();
    let pool = Database::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database_user,
        config.database_password,
        config.database_host,
        config.database_port,
        config.database_name,
    ))
    .await;
    if cfg!(debug_assertions) {
        run_migrations(&pool.pool).await?;
    }
    let app_state = Arc::new(AppState::new(pool, config.clone()));

    #[cfg(feature = "contms-provider")]
    tokio::spawn(contms_products_sync_task(app_state.clone()));

    tokio::spawn(broadcasts_task(app_state.clone()));
    tokio::spawn(pending_payments_task(app_state.clone()));

    let app = create_app(app_state.clone())
        .merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", ApiDoc::openapi()));

    let listener_address = format!("0.0.0.0:{}", config.backend_port);
    tracing::info!("listening on {}", listener_address);

    let listener = tokio::net::TcpListener::bind(listener_address).await?;

    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal(app_state));

    if let Err(e) = server.await {
        tracing::error!(error = %e, "server error");
    }

    Ok(())
}

async fn shutdown_signal(_state: Arc<AppState>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("received CTRL+C, shutting down...");
        }
        _ = terminate => {
            tracing::info!("received terminate signal, shutting down...");
        }
    };
}
