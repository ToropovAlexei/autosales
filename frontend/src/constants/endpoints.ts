export const ENDPOINTS = {
  USERS_ME: "me",
  USERS_ME_REFERRAL_SETTINGS: "me/referral-settings",
  REFERRALS: "referrals",
  REFERRAL_BOTS_ADMIN: "referrals/admin-list",
  CATEGORIES: "categories",
  PRODUCTS: "products",
  TRANSACTIONS: "transactions",
  STOCK_MOVEMENTS: "stock/movements",
  DASHBOARD_STATS: "dashboard/stats",
  DASHBOARD_TIME_SERIES: "dashboard/time-series",
  DASHBOARD_TOP_PRODUCTS: "dashboard/top-products",
  DASHBOARD_SALES_BY_CATEGORY: "dashboard/sales-by-category",
  BOT_USERS: "admin/bot-users",
  ORDERS: "orders",
  SET_BOT_PRIMARY: "referrals/:id/set-primary",
};

export const ENDPOINT_UPDATE_PUT_EXCEPTIONS = new Set<string>([
  ENDPOINTS.USERS_ME_REFERRAL_SETTINGS,
  ENDPOINTS.REFERRALS,
  ENDPOINTS.SET_BOT_PRIMARY,
  ENDPOINTS.CATEGORIES,
]);
