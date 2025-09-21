export interface CategoryResponse {
  id: number;
  name: string;
  parent_id?: number;
  sub_categories: CategoryResponse[];
}