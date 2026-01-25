use serde::Serialize;
use utoipa::{ToResponse, ToSchema};

#[derive(Debug, ToSchema, ToResponse, Serialize)]
pub struct ListResponse<T>
where
    T: ToSchema,
{
    pub items: Vec<T>,
    pub total: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    // A dummy struct to use with ListResponse for testing
    #[derive(Debug, Serialize, Deserialize, ToSchema)]
    struct DummyItem {
        id: i32,
        name: String,
    }

    #[test]
    fn test_list_response_serialization() {
        let items = vec![
            DummyItem {
                id: 1,
                name: "Item 1".to_string(),
            },
            DummyItem {
                id: 2,
                name: "Item 2".to_string(),
            },
        ];
        let total = 2;

        let list_response = ListResponse { items, total };

        let serialized = serde_json::to_string(&list_response).unwrap();
        let expected = r#"{"items":[{"id":1,"name":"Item 1"},{"id":2,"name":"Item 2"}],"total":2}"#;

        assert_eq!(serialized, expected);
    }
}
