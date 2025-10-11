export * from './common';
export * from './role';

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
  price: number;
  stock: number;
  type: "item" | "subscription";
  subscription_period_days: number;
  provider?: string;
  external_id?: string;
}