"use client";

import React, { useState, useMemo } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { ICategory } from "@/types";
import { RichTreeView } from "@mui/x-tree-view/RichTreeView";
import { TreeItem, TreeItemProps } from "@mui/x-tree-view/TreeItem";
import { categoriesToList, flattenCategoriesForSelect } from "./utils";
import classes from "./styles.module.css";
import { CategoryForm } from "./CategoryForm";
import { Button, IconButton } from "@mui/material";
import EditIcon from "@mui/icons-material/Edit";
import DeleteIcon from "@mui/icons-material/Delete";
import AddIcon from "@mui/icons-material/Add";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { PageLayout } from "@/components/PageLayout";
import { CONFIG } from "../../../../config";
import { ConfirmModal } from "@/components";

export default function CategoriesPage() {
  const queryClient = useQueryClient();
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [selectedCategory, setSelectedCategory] =
    useState<Partial<ICategory> | null>(null);
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);

  const { data: categories, isPending } = useList<ICategory>({
    endpoint: ENDPOINTS.CATEGORIES,
  });

  const mutation = useMutation({
    mutationFn: (payload: {
      id?: number;
      name: string;
      parent_id?: number;
      image_id?: string;
    }) => {
      const params = {
        url: ENDPOINTS.CATEGORIES,
        params: {
          name: payload.name,
          parent_id: payload.parent_id,
          image_id: payload.image_id,
        },
      };
      if (payload.id) {
        return dataLayer.update({
          ...params,
          id: payload.id,
        });
      }
      return dataLayer.create(params);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.CATEGORIES),
      });
      setIsDialogOpen(false);
      setSelectedCategory(null);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: number) =>
      dataLayer.delete({ url: ENDPOINTS.CATEGORIES, id }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.CATEGORIES),
      });
    },
  });

  const openDialog = (category?: Partial<ICategory>) => {
    if (category) {
      setSelectedCategory(category);
    }
    setIsDialogOpen(true);
  };

  const flattenedCategories = useMemo(
    () =>
      categories?.data
        ? [
            { value: 0, label: "Нет (корневая категория)" },
            ...flattenCategoriesForSelect(categories.data),
          ]
        : [],
    [categories]
  );

  const categoriesTree = useMemo(
    () => categoriesToList(categories?.data || []),
    [categories]
  );

  const handleDelete = (event: React.MouseEvent) => {
    event.stopPropagation();
    if (!selectedCategory?.id) {
      return;
    }
    deleteMutation.mutate(selectedCategory.id);
  };

  const findCategoryById = (
    categories: ICategory[],
    id: number
  ): ICategory | null => {
    for (const category of categories) {
      if (category.id === id) {
        return category;
      }
      if (category.sub_categories) {
        const found = findCategoryById(category.sub_categories, id);
        if (found) {
          return found;
        }
      }
    }
    return null;
  };

  const CustomTreeItem = React.forwardRef(function CustomTreeItem(
    props: TreeItemProps,
    ref: React.Ref<HTMLLIElement>
  ) {
    const { itemId, label, ...other } = props;
    const categoryId = parseInt(itemId, 10);
    const category = findCategoryById(categories?.data || [], categoryId);

    const handleEdit = (event: React.MouseEvent) => {
      event.stopPropagation();
      if (category) {
        openDialog(category);
      }
    };

    const handleAddSubCategory = (event: React.MouseEvent) => {
      event.stopPropagation();
      openDialog({ parent_id: categoryId });
    };

    return (
      <TreeItem
        {...other}
        ref={ref}
        itemId={itemId}
        label={
          <div className={classes.treeItemLabel}>
            <div className={classes.labelContainer}>
              {category?.image_id && (
                <img
                  src={`${CONFIG.IMAGES_URL}/${category.image_id}`}
                  className={classes.categoryImage}
                />
              )}
              <span className={classes.treeItemLabelText}>{label}</span>
            </div>
            <div className={classes.actions}>
              <IconButton
                aria-label="add"
                onClick={handleAddSubCategory}
                size="small"
              >
                <AddIcon />
              </IconButton>
              <IconButton aria-label="edit" onClick={handleEdit} size="small">
                <EditIcon />
              </IconButton>
              <IconButton
                aria-label="delete"
                onClick={() => {
                  if (category) {
                    setSelectedCategory(category);
                  }
                  setIsConfirmOpen(true);
                }}
                size="small"
              >
                <DeleteIcon />
              </IconButton>
            </div>
          </div>
        }
      />
    );
  });

  if (isPending) return <div>Загрузка...</div>;

  return (
    <PageLayout title="Категории">
      <Button variant="contained" onClick={() => openDialog()} sx={{ mb: 2 }}>
        Добавить категорию
      </Button>
      <RichTreeView
        items={categoriesTree}
        className={classes.tree}
        itemChildrenIndentation={24}
        slots={{ item: CustomTreeItem }}
      />
      {isDialogOpen && (
        <CategoryForm
          open={isDialogOpen}
          onClose={() => {
            setIsDialogOpen(false);
            setSelectedCategory(null);
          }}
          onConfirm={mutation.mutate}
          categories={flattenedCategories}
          defaultValues={selectedCategory || undefined}
        />
      )}
      <ConfirmModal
        open={isConfirmOpen}
        contentText={`Вы уверены, что хотите удалить категорию ${selectedCategory?.name}?`}
        onClose={() => setIsConfirmOpen(false)}
        onConfirm={handleDelete}
        title="Вы уверены?"
      />
    </PageLayout>
  );
}
