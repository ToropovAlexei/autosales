use std::fmt;

use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

#[derive(FromRow, Debug)]
pub struct PermissionRow {
    pub id: i64,
    pub name: String,
    pub group: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    // 🔐 RBAC
    RbacManage,

    // 📊 Dashboard
    DashboardRead,

    // 🛍️ Products
    ProductsCreate,
    ProductsRead,
    ProductsUpdate,
    ProductsDelete,

    // 🗂️ Categories
    CategoriesCreate,
    CategoriesRead,
    CategoriesUpdate,
    CategoriesDelete,

    // 📦 Stock
    StockCreate,
    StockRead,

    // 📦 Orders
    OrdersRead,

    // 👥 Admin users
    AdminUsersCreate,
    AdminUsersRead,
    AdminUsersUpdate,
    AdminUsersDelete,

    // 👤 Customers
    CustomersRead,
    CustomersUpdate,

    // 🖼️ Images
    ImagesCreate,
    ImagesRead,
    ImagesUpdate,
    ImagesDelete,

    // 💰 Finances
    TransactionsRead,
    StoreBalanceRead,
    StoreBalanceDeposit,
    StoreBalanceWithdraw,

    // 📋 Invoices
    InvoicesRead,

    // 🤖 Bots
    BotsCreate,
    BotsRead,
    BotsUpdate,
    BotsDelete,

    // ⚙️ Settings
    SettingsRead,
    SettingsEdit,
    PricingRead,
    PricingEdit,

    // 📢 Broadcast
    BroadcastCreate,
    BroadcastRead,

    // 📝 Audit
    AuditLogRead,
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            // 🔐 RBAC
            Self::RbacManage => "rbac:manage",

            // 📊 Dashboard
            Self::DashboardRead => "dashboard:read",

            // 🛍️ Продукты
            Self::ProductsCreate => "products:create",
            Self::ProductsRead => "products:read",
            Self::ProductsUpdate => "products:update",
            Self::ProductsDelete => "products:delete",

            // 🗂️ Категории
            Self::CategoriesCreate => "categories:create",
            Self::CategoriesRead => "categories:read",
            Self::CategoriesUpdate => "categories:update",
            Self::CategoriesDelete => "categories:delete",

            // 📦 Склад
            Self::StockCreate => "stock:create",
            Self::StockRead => "stock:read",

            // 📦 Заказы
            Self::OrdersRead => "orders:read",

            // 👥 Администраторы
            Self::AdminUsersCreate => "admin_users:create",
            Self::AdminUsersRead => "admin_users:read",
            Self::AdminUsersUpdate => "admin_users:update",
            Self::AdminUsersDelete => "admin_users:delete",

            // 👤 Покупатели
            Self::CustomersRead => "customers:read",
            Self::CustomersUpdate => "customers:update",

            // 🖼️ Изображения
            Self::ImagesCreate => "images:create",
            Self::ImagesRead => "images:read",
            Self::ImagesUpdate => "images:update",
            Self::ImagesDelete => "images:delete",

            // 💰 Финансы
            Self::TransactionsRead => "transactions:read",
            Self::StoreBalanceRead => "store_balance:read",
            Self::StoreBalanceDeposit => "store_balance:deposit",
            Self::StoreBalanceWithdraw => "store_balance:withdraw",

            // 📋 Инвойсы
            Self::InvoicesRead => "invoices:read",

            // 🤖 Боты
            Self::BotsCreate => "bots:create",
            Self::BotsRead => "bots:read",
            Self::BotsUpdate => "bots:update",
            Self::BotsDelete => "bots:delete",

            // ⚙️ Настройки
            Self::SettingsRead => "settings:read",
            Self::SettingsEdit => "settings:edit",
            Self::PricingRead => "pricing:read",
            Self::PricingEdit => "pricing:edit",

            // 📢 Рассылки
            Self::BroadcastCreate => "broadcast:create",
            Self::BroadcastRead => "broadcast:read",

            // 📝 Аудит
            Self::AuditLogRead => "audit_log:read",
        };
        f.write_str(s)
    }
}
