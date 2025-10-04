import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { ICategory } from "@/types";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

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

// Рекурсивно ищет имя категории по ID в дереве
export const findCategoryNameById = (
  categories: ICategory[],
  id: number
): string | null => {
  for (const category of categories) {
    if (category.id === id) {
      return category.name;
    }
    if (category.sub_categories && category.sub_categories.length > 0) {
      const found = findCategoryNameById(category.sub_categories, id);
      if (found) {
        return found;
      }
    }
  }
  return null;
};
