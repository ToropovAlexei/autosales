use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{
    errors::{api::ApiError, auth::AuthError},
    models::permission::Permission,
    services::auth::AuthUser,
    state::AppState,
};

impl<P> FromRequestParts<Arc<AppState>> for RequirePermission<P>
where
    P: PermissionMarker + Default,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;

        let has_permission = state
            .auth_service
            .has_permission(user.id, P::PERMISSION)
            .await
            .map_err(|e: AuthError| ApiError::AuthenticationError(e.to_string()))?;

        if !has_permission {
            return Err(ApiError::AuthorizationError(
                "Missing permission".to_string(),
            ));
        }

        Ok(RequirePermission(P::default()))
    }
}

pub struct RequirePermission<P>(pub P)
where
    P: PermissionMarker;

pub trait PermissionMarker {
    const PERMISSION: Permission;
}

macro_rules! permission_marker {
    ($($variant:ident),* $(,)?) => {
        $(
            paste::paste! {
                #[derive(Debug, Clone, Copy, Default)]
                pub struct [<$variant Permission>];

                impl PermissionMarker for [<$variant Permission>] {
                    const PERMISSION: Permission = Permission::$variant;
                }
            }
        )*
    };
}

permission_marker! {
    RbacManage,
    DashboardRead,
    ProductsCreate, ProductsRead, ProductsUpdate, ProductsDelete,
    CategoriesCreate, CategoriesRead, CategoriesUpdate, CategoriesDelete,
    StockCreate, StockRead,
    OrdersRead,
    AdminUsersCreate, AdminUsersRead, AdminUsersUpdate, AdminUsersDelete,
    CustomersRead, CustomersUpdate,
    ImagesCreate, ImagesRead, ImagesUpdate, ImagesDelete,
    TransactionsRead,
    StoreBalanceRead, StoreBalanceDeposit, StoreBalanceWithdraw,
    InvoicesRead,
    BotsCreate, BotsRead, BotsUpdate, BotsDelete,
    SettingsRead, SettingsEdit,
    PricingRead, PricingEdit,
    BroadcastCreate, BroadcastRead,
    AuditLogRead,
}
