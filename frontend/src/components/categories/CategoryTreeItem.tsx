import { Button } from "@/components/ui/button";
import { ICategory } from "@/types";

interface CategoryTreeItemProps {
  category: ICategory;
  onEdit: (category: ICategory) => void;
  onDelete: (id: number) => void;
  onAddSubCategory: (parentId: number) => void;
}

export function CategoryTreeItem({
  category,
  onEdit,
  onDelete,
  onAddSubCategory,
}: CategoryTreeItemProps) {
  return (
    <li className="pl-4 border-l border-gray-200 dark:border-gray-700 py-1">
      <div className="flex items-center justify-between group">
        <span>{category.name}</span>
        <div className="opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onAddSubCategory(category.id)}
          >
            + Добавить
          </Button>
          <Button variant="ghost" size="sm" onClick={() => onEdit(category)}>
            ✏️
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onDelete(category.id)}
          >
            🗑️
          </Button>
        </div>
      </div>
      {category.sub_categories && category.sub_categories.length > 0 && (
        <ul className="pl-4 mt-2">
          {category.sub_categories.map((subCategory) => (
            <CategoryTreeItem
              key={subCategory.id}
              category={subCategory}
              onEdit={onEdit}
              onDelete={onDelete}
              onAddSubCategory={onAddSubCategory}
            />
          ))}
        </ul>
      )}
    </li>
  );
}
