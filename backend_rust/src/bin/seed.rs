use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};

use backend_rust::{
    bin::{
        assign_role_to_admin_user, create_admin_user_if_not_exists,
        create_admin_user_role_if_not_exists,
    },
    config::Config,
    db::Database,
    infrastructure::repositories::{
        admin_user::AdminUserRepository,
        audit_log::AuditLogRepository,
        category::{CategoryRepository, CategoryRepositoryTrait},
        products::ProductRepository,
        role::RoleRepository,
        settings::SettingsRepository,
        stock_movement::StockMovementRepository,
        user_role::UserRoleRepository,
    },
    init_tracing,
    middlewares::context::RequestContext,
    models::{
        category::NewCategory,
        common::{OrderDir, Pagination},
        product::ProductListQuery,
    },
    run_migrations,
    services::{
        audit_log::AuditLogService,
        category::{CategoryService, CategoryServiceTrait},
        product::{CreateProductCommand, ProductService, ProductServiceTrait},
        topt_encryptor::TotpEncryptor,
    },
    state::AppState,
};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use shared_dtos::product::ProductType;
use uuid::Uuid;

type CategoryServiceShortType =
    CategoryService<CategoryRepository, AuditLogService<AuditLogRepository>>;

type ProductServiceShortType = ProductService<
    ProductRepository,
    StockMovementRepository,
    AuditLogService<AuditLogRepository>,
    SettingsRepository,
    CategoryServiceShortType,
>;

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
    run_migrations(&pool.pool).await?;
    let db_pool = Arc::new(pool.get_pool().clone());
    let _app_state = Arc::new(AppState::new(pool.clone(), config.clone()));
    let totp_encryptor = Arc::new(
        TotpEncryptor::new(&config.totp_encode_secret.clone())
            .expect("Failed to init totp_encryptor"),
    );
    let admin_user_repo = Arc::new(AdminUserRepository::new(db_pool.clone()));
    let role_repo = Arc::new(RoleRepository::new(db_pool.clone()));
    let user_role_repo = Arc::new(UserRoleRepository::new(db_pool.clone()));
    let category_repo = Arc::new(CategoryRepository::new(db_pool.clone()));
    let product_repo = Arc::new(ProductRepository::new(db_pool.clone()));
    let stock_movement_repo = Arc::new(StockMovementRepository::new(db_pool.clone()));
    let audit_log_repo = Arc::new(AuditLogRepository::new(db_pool.clone()));
    let audit_log_service = Arc::new(AuditLogService::new(audit_log_repo.clone()));
    let settings_repo = Arc::new(SettingsRepository::new(db_pool.clone()));
    let category_service = Arc::new(CategoryService::new(
        category_repo.clone(),
        audit_log_service.clone(),
    ));
    let product_service = Arc::new(ProductService::new(
        product_repo,
        stock_movement_repo,
        settings_repo,
        audit_log_service.clone(),
        category_service.clone(),
    ));

    let admin_id = create_admin_user_if_not_exists(
        &admin_user_repo,
        &totp_encryptor,
        "admin_dev".to_string(),
        Some("password".to_string()),
        Some("QO4C6IF3RRNNUXLKAIVLOQPVYM5W3XEV".to_string()),
    )
    .await;
    println!("Admin Id: {}", admin_id);
    let admin_role_id = create_admin_user_role_if_not_exists(&role_repo, "admin").await;
    println!("Admin role Id: {}", admin_role_id);
    assign_role_to_admin_user(admin_id, admin_role_id, &user_role_repo).await;
    println!("Admin user role assigned");
    seed_categories(&category_repo).await;
    println!("Test categories created");
    seed_products(&product_service, &category_service).await;

    Ok(())
}

async fn seed_categories(category_repo: &Arc<CategoryRepository>) {
    println!("🌱 Seeding test categories...");

    let existing = category_repo.get_list().await.unwrap();
    let existing_names: std::collections::HashSet<_> =
        existing.iter().map(|c| c.name.as_str()).collect();

    let create_if_not_exists = async |name: &str, parent_id: Option<i64>| {
        if !existing_names.contains(name) {
            println!("  ➕ {}", name);
            let cat = category_repo
                .create(NewCategory {
                    name: name.to_string(),
                    parent_id,
                    image_id: None,
                    created_by: 1, // System
                })
                .await
                .unwrap();
            Some(cat.id)
        } else {
            println!("  ✅ {} (уже есть)", name);
            existing.iter().find(|c| c.name == name).map(|c| c.id)
        }
    };

    // --- 1. Корневые категории ---
    let electronics_id = create_if_not_exists("Электроника", None).await.unwrap();
    let books_id = create_if_not_exists("Книги", None).await.unwrap();
    let clothes_id = create_if_not_exists("Одежда и обувь", None).await.unwrap();
    let home_id = create_if_not_exists("Дом и сад", None).await.unwrap();
    let sport_id = create_if_not_exists("Спорт и отдых", None).await.unwrap();

    // --- 2. Электроника ---
    let phones_id = create_if_not_exists("Смартфоны", Some(electronics_id))
        .await
        .unwrap();
    create_if_not_exists("Ноутбуки", Some(electronics_id))
        .await
        .unwrap();
    create_if_not_exists("Наушники", Some(electronics_id))
        .await
        .unwrap();

    // --- 3. 📱 Смартфоны ---
    create_if_not_exists("Android", Some(phones_id))
        .await
        .unwrap();
    create_if_not_exists("iOS", Some(phones_id)).await.unwrap();

    // --- 4. Книги ---
    create_if_not_exists("Художественная литература", Some(books_id))
        .await
        .unwrap();
    create_if_not_exists("Научная литература", Some(books_id))
        .await
        .unwrap();
    create_if_not_exists("Детские книги", Some(books_id))
        .await
        .unwrap();

    // --- 5. Одежда и обувь ---
    let mens_id = create_if_not_exists("Мужская одежда", Some(clothes_id))
        .await
        .unwrap();
    create_if_not_exists("Женская одежда", Some(clothes_id))
        .await
        .unwrap();
    create_if_not_exists("Обувь", Some(clothes_id))
        .await
        .unwrap();

    // --- 6. Мужская одежда → 3-й уровень ---
    create_if_not_exists("Футболки", Some(mens_id))
        .await
        .unwrap();
    create_if_not_exists("Джинсы", Some(mens_id)).await.unwrap();

    // --- 7. Дом и сад ---
    create_if_not_exists("Мебель", Some(home_id)).await.unwrap();
    create_if_not_exists("Освещение", Some(home_id))
        .await
        .unwrap();
    create_if_not_exists("Садовый инвентарь", Some(home_id))
        .await
        .unwrap();

    // --- 8. Спорт и отдых ---
    create_if_not_exists("Фитнес", Some(sport_id))
        .await
        .unwrap();
    create_if_not_exists("Туризм", Some(sport_id))
        .await
        .unwrap();

    println!("✅ Categories seeded successfully!");
}

pub async fn seed_products(
    product_service: &Arc<ProductServiceShortType>,
    category_service: &Arc<
        CategoryService<CategoryRepository, AuditLogService<AuditLogRepository>>,
    >,
) {
    println!("🌱 Seeding test products...");

    let existing = product_service
        .get_list(ProductListQuery {
            pagination: Pagination {
                page: 1,
                page_size: 1000,
            },
            filters: vec![],
            order_by: None,
            order_dir: OrderDir::Desc,
        })
        .await
        .unwrap();
    let existing_names: std::collections::HashSet<_> =
        existing.items.iter().map(|p| p.name.as_str()).collect();

    let categories = category_service.get_list().await.unwrap();
    let category_by_name: std::collections::HashMap<_, _> =
        categories.into_iter().map(|c| (c.name, c.id)).collect();

    let get_category_id = |name: &str| -> i64 {
        *category_by_name.get(name).unwrap_or_else(|| {
            panic!(
                "Category '{}' not found! Did you seed categories first?",
                name
            )
        })
    };

    let ctx = RequestContext {
        ip_address: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
        user_agent: Some("seed_script".to_string()),
        request_id: Uuid::new_v4(),
    };

    let create_if_not_exists = |name: String,
                                base_price: f64,
                                category_name: String,
                                initial_stock: Option<i64>,
                                product_type: ProductType| {
        let product_service = Arc::clone(product_service);
        let ctx = ctx.clone();
        let name = name.to_string();
        let category_name = category_name.to_string();
        let existing_names = existing_names.clone();
        async move {
            if !existing_names.contains(&name.as_str()) {
                println!("  ➕ {}", name);
                let cmd = CreateProductCommand {
                    name: name.to_string(),
                    base_price: Decimal::from_f64(base_price).unwrap(),
                    category_id: get_category_id(category_name.as_str()),
                    image_id: None,
                    r#type: product_type,
                    subscription_period_days: None,
                    details: None,
                    fulfillment_text: None,
                    fulfillment_image_id: None,
                    provider_name: "internal".to_string(),
                    external_id: None,
                    created_by: 1, // System user
                    initial_stock,
                    ctx: Some(ctx),
                };
                product_service.create(cmd).await.unwrap();
            } else {
                println!("  ✅ {} (уже есть)", name);
            }
        }
    };

    // --- Электроника ---

    create_if_not_exists(
        "iPhone 15 Pro".to_string(),
        1299.99,
        "iOS".to_string(),
        Some(10),
        ProductType::Item,
    )
    .await;

    create_if_not_exists(
        "Samsung Galaxy S24".to_string(),
        999.99,
        "Android".to_string(),
        Some(15),
        ProductType::Item,
    )
    .await;

    create_if_not_exists(
        "AirPods Pro".to_string(),
        249.99,
        "Наушники".to_string(),
        Some(30),
        ProductType::Item,
    )
    .await;

    create_if_not_exists(
        "MacBook Air M2".to_string(),
        1199.99,
        "Ноутбуки".to_string(),
        Some(5),
        ProductType::Item,
    )
    .await;

    // --- Книги ---

    create_if_not_exists(
        "Мастер и Маргарита".to_string(),
        15.99,
        "Художественная литература".to_string(),
        Some(100),
        ProductType::Item,
    )
    .await;

    create_if_not_exists(
        "Алгоритмы. Построение и анализ".to_string(),
        65.00,
        "Научная литература".to_string(),
        Some(40),
        ProductType::Item,
    )
    .await;

    create_if_not_exists(
        "Гарри Поттер и философский камень".to_string(),
        18.50,
        "Детские книги".to_string(),
        Some(80),
        ProductType::Item,
    )
    .await;

    // --- Одежда ---

    create_if_not_exists(
        "Футболка хлопковая (M, белая)".to_string(),
        19.99,
        "Футболки".to_string(),
        Some(50),
        ProductType::Item,
    )
    .await;

    create_if_not_exists(
        "Джинсы классические (32, синие)".to_string(),
        79.99,
        "Джинсы".to_string(),
        Some(25),
        ProductType::Item,
    )
    .await;

    // --- Дом и сад ---

    create_if_not_exists(
        "Светодиодный светильник потолочный".to_string(),
        45.00,
        "Освещение".to_string(),
        Some(20),
        ProductType::Item,
    )
    .await;

    create_if_not_exists(
        "Лопата садовая стальная".to_string(),
        29.99,
        "Садовый инвентарь".to_string(),
        Some(15),
        ProductType::Item,
    )
    .await;

    // --- Спорт ---

    create_if_not_exists(
        "Гантели 5 кг (пара)".to_string(),
        39.99,
        "Фитнес".to_string(),
        Some(12),
        ProductType::Item,
    )
    .await;

    create_if_not_exists(
        "Палатка туристическая 2-местная".to_string(),
        199.99,
        "Туризм".to_string(),
        Some(8),
        ProductType::Item,
    )
    .await;

    println!("✅ Products seeded successfully!");
}
