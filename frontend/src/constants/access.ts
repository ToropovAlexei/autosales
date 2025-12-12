import { PermissionName, AppRoute } from "@/types";

export const ROUTES_ACCESS_MAP: Record<AppRoute, PermissionName> = {
  [AppRoute.Dashboard]: PermissionName.DashboardRead,
  [AppRoute.Categories]: PermissionName.CategoriesRead,
  [AppRoute.Products]: PermissionName.ProductsRead,
  [AppRoute.BotUsers]: PermissionName.UsersRead,
  [AppRoute.Transactions]: PermissionName.TransactionsRead,
  [AppRoute.Orders]: PermissionName.OrdersRead,
  [AppRoute.Stock]: PermissionName.StockRead,
  [AppRoute.Bots]: PermissionName.ReferralsRead,
  [AppRoute.Roles]: PermissionName.RbacManage,
  [AppRoute.Users]: PermissionName.RbacManage,
  [AppRoute.Images]: PermissionName.ImagesUpload,
  [AppRoute.AuditLog]: PermissionName.AuditLogRead,
  [AppRoute.Settings]: PermissionName.SettingsRead,
  [AppRoute.Pricing]: PermissionName.PricingRead,
  [AppRoute.Balance]: PermissionName.StoreBalanceManage,
};
