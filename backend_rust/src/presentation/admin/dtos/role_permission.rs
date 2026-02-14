#[cfg(test)]
mod tests {
    use shared_dtos::role_permission::UpdateRolePermissionsAdminRequest;

    #[test]
    fn test_update_role_permissions_request_creation() {
        let req = UpdateRolePermissionsAdminRequest {
            added: vec![1, 2, 3],
            removed: vec![4, 5],
        };

        assert_eq!(req.added, vec![1, 2, 3]);
        assert_eq!(req.removed, vec![4, 5]);
    }

    #[test]
    fn test_update_role_permissions_request_serialization_deserialization() {
        let req = UpdateRolePermissionsAdminRequest {
            added: vec![10, 20],
            removed: vec![30],
        };

        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: UpdateRolePermissionsAdminRequest =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.added, req.added);
        assert_eq!(deserialized.removed, req.removed);
    }
}
