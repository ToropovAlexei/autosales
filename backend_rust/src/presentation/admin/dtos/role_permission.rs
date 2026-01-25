use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "UpdateRolePermissions")]
pub struct UpdateRolePermissionsRequest {
    pub added: Vec<i64>,
    pub removed: Vec<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_role_permissions_request_creation() {
        let req = UpdateRolePermissionsRequest {
            added: vec![1, 2, 3],
            removed: vec![4, 5],
        };

        assert_eq!(req.added, vec![1, 2, 3]);
        assert_eq!(req.removed, vec![4, 5]);
    }

    #[test]
    fn test_update_role_permissions_request_serialization_deserialization() {
        let req = UpdateRolePermissionsRequest {
            added: vec![10, 20],
            removed: vec![30],
        };

        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: UpdateRolePermissionsRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.added, req.added);
        assert_eq!(deserialized.removed, req.removed);
    }
}
