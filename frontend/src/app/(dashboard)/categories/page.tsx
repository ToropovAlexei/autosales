"use client";

import { useState, useMemo } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import api from "@/lib/api";
import { List } from "@/components/List";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { CategoryResponse } from "@/types";
import { CategoryTreeItem } from "@/components/categories/CategoryTreeItem";

// Helper function to flatten the category tree for the select dropdown
const flattenCategoriesForSelect = (
  categories: CategoryResponse[],
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

export default function CategoriesPage() {
  const queryClient = useQueryClient();
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [dialogMode, setDialogMode] = useState<"add" | "edit">("add");
  const [selectedCategory, setSelectedCategory] =
    useState<Partial<CategoryResponse> | null>(null);

  const { data: categories, isPending } = useList<CategoryResponse>({
    endpoint: ENDPOINTS.CATEGORIES,
  });

  const mutation = useMutation({
    mutationFn: (payload: {
      id?: number;
      name: string;
      parent_id?: number;
    }) => {
      if (dialogMode === "edit" && payload.id) {
        return api.put(`/categories/${payload.id}`, {
          name: payload.name,
          parent_id: payload.parent_id,
        });
      } else {
        return api.post("/categories", {
          name: payload.name,
          parent_id: payload.parent_id,
        });
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["categories"] });
      setIsDialogOpen(false);
      setSelectedCategory(null);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: number) => api.delete(`/categories/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["categories"] });
    },
  });

  const openDialog = (
    mode: "add" | "edit",
    category?: CategoryResponse,
    parentId?: number
  ) => {
    setDialogMode(mode);
    if (mode === "edit" && category) {
      setSelectedCategory(category);
    } else if (mode === "add") {
      setSelectedCategory({ name: "", parent_id: parentId });
    } else {
      setSelectedCategory({ name: "" });
    }
    setIsDialogOpen(true);
  };

  const handleSubmit = () => {
    if (
      selectedCategory &&
      selectedCategory.name &&
      selectedCategory.name?.trim() !== ""
    ) {
      mutation.mutate({
        id: selectedCategory.id,
        name: selectedCategory.name,
        parent_id: selectedCategory.parent_id,
      });
    }
  };

  const flattenedCategories = useMemo(
    () => (categories?.data ? flattenCategoriesForSelect(categories.data) : []),
    [categories]
  );

  if (isPending) return <div>Loading...</div>;

  return (
    <>
      <List
        title="Категории"
        addButton={
          <Button onClick={() => openDialog("add")}>Добавить категорию</Button>
        }
      >
        <div className="p-4">
          <ul>
            {categories?.data?.map((category) => (
              <CategoryTreeItem
                key={category.id}
                category={category}
                onEdit={(cat) => openDialog("edit", cat)}
                onDelete={deleteMutation.mutate}
                onAddSubCategory={(parentId) =>
                  openDialog("add", undefined, parentId)
                }
              />
            ))}
          </ul>
        </div>
      </List>

      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent className="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle>
              {dialogMode === "add" ? "Добавить" : "Редактировать"} категорию
            </DialogTitle>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="grid items-center grid-cols-4 gap-4">
              <Label htmlFor="name" className="text-right">
                Название
              </Label>
              <Input
                id="name"
                value={selectedCategory?.name || ""}
                onChange={(e) =>
                  setSelectedCategory((cat) => ({
                    ...cat,
                    name: e.target.value,
                  }))
                }
                className="col-span-3"
              />
            </div>
            <div className="grid items-center grid-cols-4 gap-4">
              <Label htmlFor="parent" className="text-right">
                Родительская категория
              </Label>
              <Select
                value={selectedCategory?.parent_id?.toString() || "0"}
                onValueChange={(value) =>
                  setSelectedCategory((cat) => ({
                    ...cat,
                    parent_id: value === "0" ? undefined : Number(value),
                  }))
                }
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Выберите родительскую категорию" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="0">Нет (корневая категория)</SelectItem>
                  {flattenedCategories.map((cat) => (
                    <SelectItem key={cat.id} value={cat.id.toString()}>
                      {cat.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button
              type="submit"
              onClick={handleSubmit}
              disabled={mutation.isPending}
            >
              {mutation.isPending ? "Сохранение..." : "Сохранить"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
