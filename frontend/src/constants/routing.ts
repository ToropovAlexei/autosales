import { AppRoute } from "@/types/routing";

export const APP_ROUTES: Record<AppRoute, string> = {
  [AppRoute.Dashboard]: "/dashboard",
  [AppRoute.Categories]: "/categories",
  [AppRoute.Products]: "/products",
  [AppRoute.BotUsers]: "/bot-users",
  [AppRoute.Transactions]: "/transactions",
  [AppRoute.Orders]: "/orders",
  [AppRoute.Stock]: "/stock",
  [AppRoute.Bots]: "/bots",
  [AppRoute.Roles]: "/roles",
  [AppRoute.Users]: "/users",
  [AppRoute.Images]: "/images",
  [AppRoute.AuditLog]: "/audit-log",
  [AppRoute.Settings]: "/settings",
  [AppRoute.Pricing]: "/pricing",
  [AppRoute.Balance]: "/balance",
};

export const ROUTE_BY_PATHNAME = Object.fromEntries(
  Object.entries(APP_ROUTES).map(([key, value]) => [
    value,
    key as unknown as AppRoute,
  ])
);
