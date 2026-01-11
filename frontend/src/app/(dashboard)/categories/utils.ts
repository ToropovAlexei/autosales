import { Category } from "@/types";
import { keyBy } from "@/utils";

export interface ICategoryList {
  id: number;
  label: string;
  children: ICategoryList[];
}

export interface ICategoryTree extends Category {
  children: ICategoryTree[];
}

export const buildCategoryTree = (categories: Category[]): ICategoryTree[] => {
  const copy = structuredClone(categories) as ICategoryTree[];
  const treeById = keyBy(copy, "id");
  const firstLevel = [] as typeof copy;

  copy.forEach((category) => {
    category.children = [];
    if (!category.parent_id) {
      firstLevel.push(category);
    }
  });

  copy.forEach((category) => {
    if (category.parent_id) {
      const parent = treeById[category.parent_id];
      if (parent) {
        parent.children.push(category);
      }
    }
  });

  return firstLevel;
};

const categoriesToListImpl = (categories: ICategoryTree[]): ICategoryList[] => {
  return categories.map((category) => {
    return {
      id: category.id,
      label: category.name,
      children: categoriesToListImpl(category.children),
    };
  });
};

export const categoriesToList = (categories: Category[]) =>
  categoriesToListImpl(buildCategoryTree(categories));

const flattenCategoriesForSelectImpl = (
  categories: ICategoryTree[],
  depth = 0
) => {
  let flatList: { value: number; label: string }[] = [];
  for (const category of categories) {
    flatList.push({
      value: category.id,
      label: "—".repeat(depth) + " " + category.name,
    });
    if (category.children && category.children.length > 0) {
      flatList = flatList.concat(
        flattenCategoriesForSelectImpl(category.children, depth + 1)
      );
    }
  }
  return flatList;
};

export const flattenCategoriesForSelect = (categories: Category[]) =>
  flattenCategoriesForSelectImpl(buildCategoryTree(categories));
