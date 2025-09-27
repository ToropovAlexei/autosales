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
  BOT_USERS: "admin/bot-users",
  SALES_OVER_TIME: "dashboard/sales-over-time",
  ORDERS: "orders",
  SET_BOT_PRIMARY: "referrals/:id/set-primary",
};

export const ENDPOINT_UPDATE_PUT_EXCEPTIONS = new Set<string>([
  ENDPOINTS.USERS_ME_REFERRAL_SETTINGS,
  ENDPOINTS.REFERRALS,
  ENDPOINTS.SET_BOT_PRIMARY,
]);
