import { PermissionName } from "@/types";

export const PERMISSION_GROUP_TRANSLATIONS: Record<string, string> = {
  RBAC: "Управление ролями",
  Dashboard: "Дашборд",
  Products: "Товары",
  Categories: "Категории",
  Orders: "Покупки",
  Users: "Пользователи",
  Settings: "Настройки",
  Images: "Изображения",
  Referrals: "Рефералы",
  Transactions: "Транзакции",
  Balance: "Баланс",
  Stock: "Склад",
  AuditLog: "Журнал аудита",
  Pricing: "Ценообразование",
  Other: "Другое",
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
  [PermissionName.UsersRead]: "Просмотр пользователей",
  [PermissionName.UsersCreate]: "Создание пользователей",
  [PermissionName.UsersUpdate]: "Редактирование пользователей",
  [PermissionName.UsersDelete]: "Удаление пользователей",
  [PermissionName.SettingsRead]: "Просмотр настроек",
  [PermissionName.SettingsEdit]: "Редактирование настроек",
  [PermissionName.ImagesRead]: "Просмотр изображений",
  [PermissionName.ImagesUpload]: "Загрузка изображений",
  [PermissionName.ImagesDelete]: "Удаление изображений",
  [PermissionName.ReferralsRead]: "Просмотр рефералов",
  [PermissionName.ReferralsUpdate]: "Редактирование рефералов",
  [PermissionName.TransactionsRead]: "Просмотр транзакций",
  [PermissionName.StoreBalanceRead]: "Просмотр баланса",
  [PermissionName.StoreBalanceManage]: "Управление балансом",
  [PermissionName.StockRead]: "Просмотр склада",
  [PermissionName.StockUpdate]: "Редактирование склада",
  [PermissionName.AuditLogRead]: "Просмотр журнала аудита",
  [PermissionName.PricingRead]: "Просмотр ценообразования",
  [PermissionName.PricingEdit]: "Редактирование ценообразования",
};

export const translatePermission = (permission: PermissionName) =>
  PERMISSION_TRANSLATIONS[permission] || permission;
