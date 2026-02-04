import { PermissionName, AppRoute } from "@/types";

export const ROUTES_ACCESS_MAP: Record<AppRoute, PermissionName> = {
  [AppRoute.Dashboard]: PermissionName.DashboardRead,
  [AppRoute.Categories]: PermissionName.CategoriesRead,
  [AppRoute.Products]: PermissionName.ProductsRead,
  [AppRoute.BotUsers]: PermissionName.AdminUsersRead,
  [AppRoute.Transactions]: PermissionName.TransactionsRead,
  [AppRoute.Orders]: PermissionName.OrdersRead,
  [AppRoute.Stock]: PermissionName.StockRead,
  [AppRoute.Bots]: PermissionName.BotsRead,
  [AppRoute.Roles]: PermissionName.RbacManage,
  [AppRoute.Users]: PermissionName.RbacManage,
  [AppRoute.Images]: PermissionName.ImagesRead,
  [AppRoute.AuditLog]: PermissionName.AuditLogRead,
  [AppRoute.Pricing]: PermissionName.PricingRead,
  [AppRoute.Balance]: PermissionName.StoreBalanceRead,
  [AppRoute.WelcomeMessages]: PermissionName.SettingsRead,
  [AppRoute.ReferralManagement]: PermissionName.SettingsRead,
  [AppRoute.Broadcasts]: PermissionName.BroadcastRead,
  [AppRoute.Operators]: PermissionName.InvoicesRead,
};
