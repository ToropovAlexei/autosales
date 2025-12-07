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
  Other: "Другое",
};

export const translatePermissionGroup = (group: string): string => {
  return PERMISSION_GROUP_TRANSLATIONS[group] || group;
};

export const PERMISSION_TRANSLATIONS: Record<string, string> = {
  "rbac:manage": "Управление ролями",
  "dashboard:read": "Просмотр дашборда",
  "products:read": "Просмотр товаров",
  "products:create": "Создание товаров",
  "products:update": "Обновление товаров",
  "products:delete": "Удаление товаров",
  "categories:read": "Просмотр категорий",
  "categories:create": "Создание категорий",
  "categories:update": "Обновление категорий",
  "categories:delete": "Удаление категорий",
  "orders:read": "Просмотр покупок",
  "users:read": "Просмотр пользователей",
  "users:create": "Создание пользователей",
  "users:update": "Обновление пользователей",
  "users:delete": "Удаление пользователей",
  "settings:read": "Просмотр настроек",
  "settings:edit": "Редактирование настроек",
  "images:read": "Просмотр изображений",
  "images:upload": "Загрузка изображений",
  "images:delete": "Удаление изображений",
  "referrals:read": "Просмотр рефералов",
  "referrals:update": "Обновление рефералов",
  "transactions:read": "Просмотр транзакций",
  "store_balance:read": "Просмотр баланса",
  "store_balance:manage": "Управление балансом",
  "stock:read": "Просмотр склада",
  "stock:update": "Обновление склада",
  "audit_log.read": "Просмотр журнала аудита",
};

export const translatePermission = (permission: string): string => {
  return PERMISSION_TRANSLATIONS[permission] || permission;
};
