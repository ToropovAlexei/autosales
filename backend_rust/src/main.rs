use std::sync::Arc;
use tokio::signal;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[cfg(feature = "contms-provider")]
use backend_rust::workers::contms_products_sync::contms_products_sync_task;
use backend_rust::{
    config::Config,
    create_app,
    db::Database,
    init_tracing,
    models::product::ProductType,
    presentation::admin::{
        dtos::{
            admin_user::{AdminUserWithRolesResponse, NewAdminUserRequest, UpdateAdminUserRequest},
            auth::{LoginStep1Request, LoginStep1Response, LoginStep2Request, LoginStep2Response},
            category::{CategoryResponse, NewCategoryRequest, UpdateCategoryRequest},
            customer::CustomerResponse,
            image::ImageResponse,
            list_response::ListResponse,
            permission::PermissionResponse,
            product::{NewProductRequest, ProductResponse, UpdateProductRequest},
            role::{NewRoleRequest, RoleResponse, UpdateRoleRequest},
            settings::{
                BotSettingsResponse, PricingSettingsResponse, UpdateBotSettingsRequest,
                UpdatePricingSettingsRequest,
            },
            stock_movement::StockMovementResponse,
        },
        handlers,
    },
    run_migrations,
    state::AppState,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::login_step1,
        handlers::auth::login_step2,
        handlers::auth::logout,
        handlers::category::create_category,
        handlers::category::delete_category,
        handlers::category::get_category,
        handlers::category::list_categories,
        handlers::category::update_category,
        handlers::product::create_product,
        handlers::product::delete_product,
        handlers::product::get_product,
        handlers::product::list_products,
        handlers::product::update_product,
        handlers::image::create_image,
        handlers::image::delete_image,
        handlers::image::list_images,
        handlers::customer::list_customers,
        handlers::customer::update_customer,
        handlers::admin_user::list_admin_users,
        handlers::admin_user::get_admin_user,
        handlers::admin_user::create_admin_user,
        handlers::admin_user::update_admin_user,
        handlers::admin_user::delete_admin_user,
        handlers::admin_user::get_admin_user_permissions,
        handlers::admin_user::update_admin_user_permissions,
        handlers::role::list_roles,
        handlers::role::create_role,
        handlers::role::update_role,
        handlers::role::delete_role,
        handlers::role::get_role_permissions,
        handlers::role::update_role_permissions,
        handlers::permission::list_permissions,
        handlers::settings::get_bot_settings,
        handlers::settings::get_pricing_settings,
        handlers::settings::update_bot_settings,
        handlers::settings::update_pricing_settings,
        handlers::me::get_me,
        handlers::me::get_me_permissions,
    ),
    components(schemas(
        CategoryResponse,
        NewCategoryRequest,
        UpdateCategoryRequest,
        ProductResponse,
        NewProductRequest,
        UpdateProductRequest,
        ProductType,
        ImageResponse,
        ListResponse<CategoryResponse>,
        ListResponse<ProductResponse>,
        ListResponse<CustomerResponse>,
        ListResponse<AdminUserWithRolesResponse>,
        ListResponse<RoleResponse>,
        ListResponse<PermissionResponse>,
        CustomerResponse,
        AdminUserWithRolesResponse,
        NewAdminUserRequest,
        UpdateAdminUserRequest,
        NewRoleRequest,
        UpdateRoleRequest,
        RoleResponse,
        PermissionResponse,
        PricingSettingsResponse,
        BotSettingsResponse,
        UpdatePricingSettingsRequest,
        UpdateBotSettingsRequest,
        LoginStep1Request,
        LoginStep1Response,
        LoginStep2Request,
        LoginStep2Response,
        StockMovementResponse,
    ))
)]
struct ApiDoc;

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
