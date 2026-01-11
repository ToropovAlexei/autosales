export * from "./common";
export * from "./permissions";
export * from "./routing";
export * from "./auth";
export * from "./admin_user";
export * from "./category";
export * from "./bot";
export * from "./audit_log";
export * from "./customer";
export * from "./image";
export * from "./invoice";
export * from "./order";
export * from "./product";
export * from "./settings";
export * from "./stock_movement";
export * from "./transaction";
export * from "./store_balance";

export interface IFilter {
  page?: number;
  page_size?: number;
  order_by?: string;
  order?: string;
  filters?: {
    field: string;
    op: string;
    value: any;
  }[];
  [key: string]: any;
}
