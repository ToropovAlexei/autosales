import { ICategory } from "@/types";

export interface ICategoryList {
  id: number;
  label: string;
  children: ICategoryList[];
}

const categoriesToListImpl = (categories: ICategory[]): ICategoryList[] => {
  return categories.map((category) => {
    return {
      id: category.id,
      label: category.name,
      children: categoriesToListImpl(category.sub_categories || []),
    };
  });
};

export const categoriesToList = (categories: ICategory[]) =>
  categoriesToListImpl(structuredClone(categories));

export const flattenCategoriesForSelect = (
  categories: ICategory[],
  depth = 0
) => {
  let flatList: { value: number; label: string }[] = [];
  for (const category of categories) {
    flatList.push({
      value: category.id,
      label: "—".repeat(depth) + " " + category.name,
    });
    if (category.sub_categories && category.sub_categories.length > 0) {
      flatList = flatList.concat(
        flattenCategoriesForSelect(category.sub_categories, depth + 1)
      );
    }
  }
  return flatList;
};
