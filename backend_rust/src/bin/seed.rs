use std::sync::Arc;

use backend_rust::{
    bin::{
        assign_role_to_admin_user, create_admin_user_if_not_exists,
        create_admin_user_role_if_not_exists,
    },
    config::Config,
    db::Database,
    infrastructure::repositories::{
        admin_user::AdminUserRepository,
        category::{CategoryRepository, CategoryRepositoryTrait},
        role::RoleRepository,
        user_role::UserRoleRepository,
    },
    init_tracing,
    models::category::NewCategory,
    run_migrations,
    services::topt_encryptor::TotpEncryptor,
    state::AppState,
};

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
