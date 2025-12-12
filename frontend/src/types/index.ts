export * from "./common";
export * from "./role";
export * from "./permissions";
export * from "./routing";

export interface IAuditLog {
  id: number;
  user_id: number;
  user_login: string;
  action: string;
  target_type: string;
  target_id: number;
  changes: any;
  status: string;
  ip_address: string;
  user_agent: string;
  created_at: string;
}

export interface ICategory {
  id: number;
  name: string;
  parent_id?: number;
  sub_categories: ICategory[];
  image_id?: string;
}

export interface IProduct {
  id: number;
  name: string;
  category_id: number;
  base_price: number;
  price: number;
  stock: number;
  type: "item" | "subscription";
  subscription_period_days: number;
  provider?: string;
  external_id?: string;
  image_id?: string;
  image_url?: string;
  fulfillment_text?: string;
  fulfillment_image_id?: string;
}

export interface IFilter {
  page?: number;
  pageSize?: number;
  orderBy?: string;
  order?: string;
  filters?: {
    field: string;
    op: string;
    value: any;
  }[];
}

export interface IStockMovement {
  id: number;
  product_id: number;
  type: "initial" | "sale" | "restock" | "return" | "adjustment";
  quantity: number;
  created_at: string;
  description: string;
  order_id?: number;
}
