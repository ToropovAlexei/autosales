use rust_decimal::prelude::ToPrimitive;
use shared_dtos::product::ProductAdminResponse;

use crate::services::product::Product;

impl From<Product> for ProductAdminResponse {
    fn from(r: Product) -> Self {
        ProductAdminResponse {
            id: r.id,
            name: r.name,
            base_price: r.base_price.to_f64().unwrap_or_default(),
            price: r.price.to_f64().unwrap_or_default(),
            stock: r.stock,
            category_id: r.category_id,
            image_id: r.image_id,
            r#type: r.r#type,
            subscription_period_days: r.subscription_period_days,
            details: r.details,
            deleted_at: r.deleted_at,
            fulfillment_text: r.fulfillment_text,
            fulfillment_image_id: r.fulfillment_image_id,
            provider_name: r.provider_name,
            external_id: r.external_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
            created_by: r.created_by,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use serde_json::json;
    use shared_dtos::product::{
        NewProductAdminRequest, ProductType, ProductsUploadResponse, UpdateProductAdminRequest,
        UploadedProductCSV,
    };
    use uuid::Uuid;
    use validator::Validate;

    fn create_test_product() -> Product {
        Product {
            id: 1,
            name: "Test Product".to_string(),
            base_price: Decimal::new(10000, 2),
            price: Decimal::new(12000, 2),
            stock: 10,
            category_id: Some(1),
            image_id: Some(Uuid::new_v4()),
            r#type: ProductType::Item,
            subscription_period_days: 30,
            details: None,
            deleted_at: None,
            fulfillment_text: Some("Here is your digital product".to_string()),
            fulfillment_image_id: Some(Uuid::new_v4()),
            provider_name: "SomeProvider".to_string(),
            external_id: Some("ext123".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: 1,
        }
    }

    #[test]
    fn test_product_response_from_product_full() {
        let product = create_test_product();
        let response: ProductAdminResponse = product.into();

        assert_eq!(response.id, 1);
        assert_eq!(response.name, "Test Product");
        assert_eq!(response.base_price, 100.0);
        assert_eq!(response.price, 120.0);
        assert_eq!(response.stock, 10);
        assert_eq!(response.category_id, Some(1));
        assert!(response.image_id.is_some());
        assert_eq!(response.r#type, ProductType::Item);
        assert_eq!(response.subscription_period_days, 30);
        assert_eq!(response.details, None);
        assert_eq!(response.deleted_at, None);
        assert_eq!(
            response.fulfillment_text,
            Some("Here is your digital product".to_string())
        );
        assert!(response.fulfillment_image_id.is_some());
        assert_eq!(response.provider_name, "SomeProvider");
        assert_eq!(response.external_id, Some("ext123".to_string()));
    }

    #[test]
    fn test_product_response_from_product_minimal() {
        let mut product = create_test_product();
        product.category_id = None;
        product.image_id = None;
        product.details = None;
        product.fulfillment_text = None;
        product.fulfillment_image_id = None;
        product.external_id = None;
        product.deleted_at = Some(Utc::now());

        let response: ProductAdminResponse = product.into();

        assert_eq!(response.category_id, None);
        assert_eq!(response.image_id, None);
        assert_eq!(response.details, None);
        assert_eq!(response.fulfillment_text, None);
        assert_eq!(response.fulfillment_image_id, None);
        assert_eq!(response.external_id, None);
        assert!(response.deleted_at.is_some());
    }

    #[test]
    fn test_new_product_request_validation() {
        // Valid request
        let req = NewProductAdminRequest {
            name: "Valid Product Name".to_string(),
            base_price: 50.00,
            category_id: 1,
            image_id: None,
            r#type: ProductType::Item,
            subscription_period_days: Some(30),
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            initial_stock: Some(100),
        };
        assert!(req.validate().is_ok());

        // Name too short
        let req = NewProductAdminRequest {
            name: "ab".to_string(),
            base_price: 50.00,
            category_id: 1,
            image_id: None,
            r#type: ProductType::Item,
            subscription_period_days: None,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            initial_stock: None,
        };
        assert!(req.validate().is_err());

        // Name too long
        let req = NewProductAdminRequest {
            name: "a".repeat(256),
            base_price: 50.00,
            category_id: 1,
            image_id: None,
            r#type: ProductType::Item,
            subscription_period_days: None,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            initial_stock: None,
        };
        assert!(req.validate().is_err());

        // Price too low
        let req = NewProductAdminRequest {
            name: "Valid Product Name".to_string(),
            base_price: 0.00,
            category_id: 1,
            image_id: None,
            r#type: ProductType::Item,
            subscription_period_days: None,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            initial_stock: None,
        };
        assert!(req.validate().is_err());

        // Price too high
        let req = NewProductAdminRequest {
            name: "Valid Product Name".to_string(),
            base_price: 1000000.00,
            category_id: 1,
            image_id: None,
            r#type: ProductType::Item,
            subscription_period_days: None,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            initial_stock: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_update_product_request_validation() {
        // Valid request: All optional fields are None
        let req = UpdateProductAdminRequest {
            name: None,
            base_price: None,
            category_id: None,
            image_id: None,
            r#type: None,
            subscription_period_days: None,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            external_id: None,
            stock: None,
        };
        assert!(req.validate().is_ok());

        // Valid request: All fields updated
        let req = UpdateProductAdminRequest {
            name: Some("Updated Name".to_string()),
            base_price: Some(75.50),
            category_id: Some(2),
            image_id: Some(Some(Uuid::new_v4())),
            r#type: Some(ProductType::Item),
            subscription_period_days: Some(60),
            details: Some(Some(json!({"size": "large"}))),
            fulfillment_text: Some(Some("New fulfillment text".to_string())),
            fulfillment_image_id: Some(Some(Uuid::new_v4())),
            external_id: Some(Some("ext456".to_string())),
            stock: Some(50),
        };
        assert!(req.validate().is_ok());

        // Valid request: Setting optional fields to None
        let req = UpdateProductAdminRequest {
            name: Some("Updated Name".to_string()),
            base_price: None,
            category_id: None,
            image_id: Some(None),
            r#type: None,
            subscription_period_days: None,
            details: Some(None),
            fulfillment_text: Some(None),
            fulfillment_image_id: Some(None),
            external_id: Some(None),
            stock: None,
        };
        assert!(req.validate().is_ok());

        // Name too short
        let req = UpdateProductAdminRequest {
            name: Some("a".to_string()),
            base_price: None,
            category_id: None,
            image_id: None,
            r#type: None,
            subscription_period_days: None,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            external_id: None,
            stock: None,
        };
        assert!(req.validate().is_err());

        // Name too long
        let req = UpdateProductAdminRequest {
            name: Some("a".repeat(256)),
            base_price: None,
            category_id: None,
            image_id: None,
            r#type: None,
            subscription_period_days: None,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            external_id: None,
            stock: None,
        };
        assert!(req.validate().is_err());

        // Price too low
        let req = UpdateProductAdminRequest {
            name: None,
            base_price: Some(0.00),
            category_id: None,
            image_id: None,
            r#type: None,
            subscription_period_days: None,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            external_id: None,
            stock: None,
        };
        assert!(req.validate().is_err());

        // Price too high
        let req = UpdateProductAdminRequest {
            name: None,
            base_price: Some(1000000.00),
            category_id: None,
            image_id: None,
            r#type: None,
            subscription_period_days: None,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            external_id: None,
            stock: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_products_upload_response_serialization() {
        let response = ProductsUploadResponse {
            created: 5,
            failed: 2,
            skipped: 1,
            errors: vec!["Error 1".to_string(), "Error 2".to_string()],
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let expected = r#"{"created":5,"failed":2,"skipped":1,"errors":["Error 1","Error 2"]}"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_uploaded_product_csv_serialization() {
        let product_csv = UploadedProductCSV {
            name: "CSV Product".to_string(),
            category: "CSV Category".to_string(),
            price: 19.99,
            initial_stock: 10,
        };

        let serialized = serde_json::to_string(&product_csv).unwrap();
        let expected =
            r#"{"name":"CSV Product","category":"CSV Category","price":19.99,"initial_stock":10}"#;
        assert_eq!(serialized, expected);
    }
}
