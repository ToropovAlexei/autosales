use rust_decimal_macros::dec;
use shared_dtos::product::ProductDetails;
use shared_dtos::product::ProductType;
use std::sync::Arc;
#[cfg(feature = "contms-provider")]
use tokio::time::{Duration, interval};

#[cfg(feature = "contms-provider")]
use crate::infrastructure::external::products::contms::ContmsProductsProviderTrait;
use crate::infrastructure::external::products::contms::dto::ContmsProxyResponse;
use crate::services::category::CategoryServiceTrait;
use crate::services::category::CreateCategoryCommand;
use crate::services::product::CreateProductCommand;
use crate::services::product::DeleteProductCommand;
use crate::services::product::ProductServiceTrait;
use crate::state::AppState;

#[cfg(feature = "contms-provider")]
pub async fn contms_products_sync_task(app_state: Arc<AppState>) {
    tracing::info!("Starting ContMs Products Sync Task");
    let mut interval = interval(Duration::from_mins(5));

    loop {
        interval.tick().await;
        tracing::info!("Running ContMs Products Sync Task...");
        let products_from_contms = match app_state.contms_products_provider.get_products().await {
            Ok(products) => products,
            Err(e) => {
                tracing::error!("Error getting ContMs products: {}", e);
                continue;
            }
        };
        tracing::info!("Got {} ContMs products", products_from_contms.len());

        let products_from_repo = match app_state
            .product_service
            .get_all_external_provider("contms")
            .await
        {
            Ok(products_from_repo) => products_from_repo,
            Err(e) => {
                tracing::error!("Error getting ContMs products from repo: {}", e);
                continue;
            }
        };

        let contms_products_set = products_from_contms
            .iter()
            .map(|p| p.name.clone())
            .collect::<std::collections::HashSet<String>>();

        let to_remove_from_repo = products_from_repo
            .iter()
            .filter(|p| {
                if let Some(p) = &p.external_id {
                    !contms_products_set.contains(p)
                } else {
                    false
                }
            })
            .map(|p| p.id)
            .collect::<Vec<i64>>();

        if !to_remove_from_repo.is_empty() {
            tracing::info!(
                "Removing {} ContMs products from repo",
                to_remove_from_repo.len()
            );
            delete_products_from_repo(&app_state, &to_remove_from_repo).await;
        }

        let repo_products_set = products_from_repo
            .iter()
            .map(|p| p.external_id.clone().unwrap_or_default())
            .collect::<std::collections::HashSet<String>>();

        let to_add_to_repo = products_from_contms
            .iter()
            .filter(|p| !repo_products_set.contains(&p.name))
            .cloned()
            .collect::<Vec<ContmsProxyResponse>>();

        if !to_add_to_repo.is_empty() {
            tracing::info!("Adding {} ContMs products to repo", to_add_to_repo.len());
            // TODO Proxy should have subcategories
            let proxy_category_id = match get_proxy_category_id(&app_state).await {
                Some(id) => id,
                None => continue,
            };

            add_products_to_repo(&app_state, &to_add_to_repo, proxy_category_id).await;
        }
    }
}

pub async fn get_proxy_category_id(app_state: &Arc<AppState>) -> Option<i64> {
    let categories = match app_state.category_service.get_list().await {
        Ok(categories) => categories,
        Err(e) => {
            tracing::error!("Error getting categories: {}", e);
            return None;
        }
    };

    let proxy_category_id = match categories.iter().find(|c| c.name == "Прокси") {
        Some(c) => c.id,
        None => match app_state
            .category_service
            .create(CreateCategoryCommand {
                ctx: None,
                name: "Прокси".to_string(),
                parent_id: None,
                image_id: None,
                created_by: 1, // System,
            })
            .await
        {
            Ok(c) => c.id,
            Err(e) => {
                tracing::error!("Error creating category: {}", e);
                return None;
            }
        },
    };

    Some(proxy_category_id)
}

pub async fn delete_products_from_repo(app_state: &Arc<AppState>, product_ids: &Vec<i64>) {
    let mut removed: i32 = 0;
    for id in product_ids {
        match app_state
            .product_service
            .delete(DeleteProductCommand {
                ctx: None,
                deleted_by: 1, // System
                id: *id,
            })
            .await
        {
            Ok(_) => removed += 1,
            Err(e) => tracing::error!("Error deleting ContMs product: {}", e),
        };
    }

    if removed != product_ids.len() as i32 {
        tracing::error!(
            "Failed to delete {} ContMs products",
            product_ids.len() - removed as usize
        );
    }
}

pub async fn add_products_to_repo(
    app_state: &Arc<AppState>,
    products: &Vec<ContmsProxyResponse>,
    category_id: i64,
) {
    let mut added: i32 = 0;
    for product in products {
        match app_state
            .product_service
            .create(CreateProductCommand {
                base_price: dec!(100), // TODO
                category_id,
                fulfillment_text: None,
                external_id: Some(product.name.clone()),
                name: product.name.clone(),
                r#type: ProductType::Subscription,
                initial_stock: None,
                provider_name: "contms".to_string(),
                subscription_period_days: Some(30), // TODO 1 month?
                fulfillment_image_id: None,
                details: Some(ProductDetails::ContMs {
                    host: product.host.clone(),
                    port: product.port,
                }),
                image_id: None,
                created_by: 1, // System
                ctx: None,
            })
            .await
        {
            Ok(_) => added += 1,
            Err(e) => tracing::error!("Error adding ContMs product: {}", e),
        };
    }

    if added != products.len() as i32 {
        tracing::error!(
            "Failed to add {} ContMs products",
            products.len() - added as usize
        );
    }
}
