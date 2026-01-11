export enum PermissionName {
  // 🔐 RBAC
  RbacManage = "rbac:manage",

  // 📊 Dashboard
  DashboardRead = "dashboard:read",

  // 🛍️ Продукты
  ProductsCreate = "products:create",
  ProductsRead = "products:read",
  ProductsUpdate = "products:update",
  ProductsDelete = "products:delete",

  // 🗂️ Категории
  CategoriesCreate = "categories:create",
  CategoriesRead = "categories:read",
  CategoriesUpdate = "categories:update",
  CategoriesDelete = "categories:delete",

  // 📦 Склад
  StockCreate = "stock:create",
  StockRead = "stock:read",

  // 📦 Заказы
  OrdersRead = "orders:read",

  // 👥 Администраторы
  AdminUsersCreate = "admin_users:create",
  AdminUsersRead = "admin_users:read",
  AdminUsersUpdate = "admin_users:update",
  AdminUsersDelete = "admin_users:delete",

  // 👤 Покупатели
  CustomersRead = "customers:read",
  CustomersUpdate = "customers:update",

  // 🖼️ Изображения
  ImagesCreate = "images:create",
  ImagesRead = "images:read",
  ImagesUpdate = "images:update",
  ImagesDelete = "images:delete",

  // 💰 Финансы
  TransactionsRead = "transactions:read",
  StoreBalanceRead = "store_balance:read",
  StoreBalanceDeposit = "store_balance:deposit",
  StoreBalanceWithdraw = "store_balance:withdraw",

  // 📋 Инвойсы
  InvoicesRead = "invoices:read",

  // 🤖 Боты
  BotsCreate = "bots:create",
  BotsRead = "bots:read",
  BotsUpdate = "bots:update",
  BotsDelete = "bots:delete",

  // ⚙️ Настройки
  SettingsRead = "settings:read",
  SettingsEdit = "settings:edit",
  PricingRead = "pricing:read",
  PricingEdit = "pricing:edit",

  // 📢 Рассылки
  BroadcastCreate = "broadcast:create",
  BroadcastRead = "broadcast:read",

  // 📝 Аудит
  AuditLogRead = "audit_log:read",
}
