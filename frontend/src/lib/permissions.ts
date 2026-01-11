import { PermissionName } from "@/types";

export const PERMISSION_GROUP_TRANSLATIONS: Record<string, string> = {
  rbac: "Управление ролями",
  dashboard: "Дашборд",
  products: "Товары",
  categories: "Категории",
  orders: "Покупки",
  admin_users: "Пользователи",
  settings: "Настройки",
  images: "Изображения",
  transactions: "Транзакции",
  store_balance: "Баланс",
  stock: "Склад",
  pricing: "Ценообразование",
  audit_log: "Журнал аудита",
  broadcast: "Рассылки",
  customers: "Покупатели",
  invoices: "Счета",
  bots: "Боты",
};

export const translatePermissionGroup = (group: string): string =>
  PERMISSION_GROUP_TRANSLATIONS[group] || group;

export const PERMISSION_TRANSLATIONS: Record<PermissionName, string> = {
  [PermissionName.RbacManage]: "Управление ролями",
  [PermissionName.DashboardRead]: "Просмотр дашборда",
  [PermissionName.ProductsRead]: "Просмотр товаров",
  [PermissionName.ProductsCreate]: "Создание товаров",
  [PermissionName.ProductsUpdate]: "Редактирование товаров",
  [PermissionName.ProductsDelete]: "Удаление товаров",
  [PermissionName.CategoriesRead]: "Просмотр категорий",
  [PermissionName.CategoriesCreate]: "Создание категорий",
  [PermissionName.CategoriesUpdate]: "Редактирование категорий",
  [PermissionName.CategoriesDelete]: "Удаление категорий",
  [PermissionName.OrdersRead]: "Просмотр покупок",
  [PermissionName.AdminUsersRead]: "Просмотр пользователей",
  [PermissionName.AdminUsersCreate]: "Создание пользователей",
  [PermissionName.AdminUsersUpdate]: "Редактирование пользователей",
  [PermissionName.AdminUsersDelete]: "Удаление пользователей",
  [PermissionName.SettingsRead]: "Просмотр настроек",
  [PermissionName.SettingsEdit]: "Редактирование настроек",
  [PermissionName.ImagesRead]: "Просмотр изображений",
  [PermissionName.ImagesUpdate]: "Редактирование изображений",
  [PermissionName.ImagesCreate]: "Загрузка изображений",
  [PermissionName.ImagesDelete]: "Удаление изображений",
  [PermissionName.TransactionsRead]: "Просмотр транзакций",
  [PermissionName.StoreBalanceRead]: "Просмотр баланса",
  [PermissionName.StoreBalanceDeposit]: "Пополнение баланса",
  [PermissionName.StoreBalanceWithdraw]: "Снятие со счета",
  [PermissionName.StockRead]: "Просмотр склада",
  [PermissionName.StockCreate]: "Редактирование склада",
  [PermissionName.AuditLogRead]: "Просмотр журнала аудита",
  [PermissionName.PricingRead]: "Просмотр ценообразования",
  [PermissionName.PricingEdit]: "Редактирование ценообразования",
  [PermissionName.BroadcastCreate]: "Создание рассылки",
  [PermissionName.BroadcastRead]: "Просмотр рассылок",
  [PermissionName.CustomersRead]: "Просмотр покупателей",
  [PermissionName.CustomersUpdate]: "Редактирование покупателей",
  [PermissionName.InvoicesRead]: "Просмотр счетов",
  [PermissionName.BotsRead]: "Просмотр ботов",
  [PermissionName.BotsCreate]: "Создание ботов",
  [PermissionName.BotsUpdate]: "Редактирование ботов",
  [PermissionName.BotsDelete]: "Удаление ботов",
};

export const translatePermission = (permission: PermissionName) =>
  PERMISSION_TRANSLATIONS[permission] || permission;
