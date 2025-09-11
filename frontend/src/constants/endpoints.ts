export const ENDPOINTS = {
  USERS_ME: "users/me",
  USERS_ME_REFERRAL_SETTINGS: "users/me/referral-settings",
  REFERRALS: "referrals",
  CATEGORIES: "categories",
  PRODUCTS: "products",
  TRANSACTIONS: "transactions",
  STOCK_MOVEMENTS: "stock/movements",
  DASHBOARD_STATS: "dashboard/stats",
  BOT_USERS: "admin/bot-users",
  SALES_OVER_TIME: "dashboard/sales-over-time",
  ORDERS: "orders",
};

export const ENDPOINT_UPDATE_PUT_EXCEPTIONS = new Set<string>([
  ENDPOINTS.USERS_ME_REFERRAL_SETTINGS,
]);
