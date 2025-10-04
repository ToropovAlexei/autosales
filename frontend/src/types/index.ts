export interface ICategory {
  id: number;
  name: string;
  parent_id?: number;
  sub_categories: ICategory[];
}
