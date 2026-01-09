pub mod models;

use async_trait::async_trait;

use crate::infrastructure::external::products::contms::models::ContmsProxy;

#[async_trait]
pub trait ContmsProductsProviderTrait {
    async fn get_products(&self) -> Vec<ContmsProxy>;
}
