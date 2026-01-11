import { ICategory } from "@/types";

// Преобразует дерево категорий в плоский список для использования в <select>
export const flattenCategoriesForSelect = (
  categories: ICategory[],
  depth = 0
) => {
  let flatList: { id: number; name: string }[] = [];
  for (const category of categories) {
    flatList.push({
      id: category.id,
      name: "—".repeat(depth) + " " + category.name,
    });
    if (category.sub_categories && category.sub_categories.length > 0) {
      flatList = flatList.concat(
        flattenCategoriesForSelect(category.sub_categories, depth + 1)
      );
    }
  }
  return flatList;
};
