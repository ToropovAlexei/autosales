"use client";

import React, { useState, useMemo } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { Category } from "@/types";
import { RichTreeView } from "@mui/x-tree-view/RichTreeView";
import { TreeItem, TreeItemProps } from "@mui/x-tree-view/TreeItem";
import { categoriesToList } from "./utils";
import classes from "./styles.module.css";
import { CategoryForm } from "./CategoryForm";
import { Button, IconButton, Skeleton } from "@mui/material";
import EditIcon from "@mui/icons-material/Edit";
import DeleteIcon from "@mui/icons-material/Delete";
import AddIcon from "@mui/icons-material/Add";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { PageLayout } from "@/components/PageLayout";
import { CONFIG } from "../../../../config";
import { ConfirmModal } from "@/components";
import { toast } from "react-toastify";
import { range } from "@/utils";

export default function CategoriesPage() {
  const queryClient = useQueryClient();
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [selectedCategory, setSelectedCategory] =
    useState<Partial<Category> | null>(null);
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);

  const { data: categories, isPending } = useList<Category>({
    endpoint: ENDPOINTS.CATEGORIES,
  });

  const deleteMutation = useMutation({
    mutationFn: (id: number) =>
      dataLayer.delete({ url: ENDPOINTS.CATEGORIES, id }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.CATEGORIES),
      });
      setIsDialogOpen(false);
      setSelectedCategory(null);
      toast.success("Категория удалена");
    },
    onError: () => {
      toast.error("Произошла ошибка");
    },
  });

  const openDialog = (category?: Partial<Category>) => {
    if (category) {
      setSelectedCategory(category);
    }
    setIsDialogOpen(true);
  };

  const categoriesTree = useMemo(
    () => categoriesToList(categories?.data || []),
    [categories],
  );

  const handleDelete = (event: React.MouseEvent) => {
    event.stopPropagation();
    if (!selectedCategory?.id) {
      return;
    }
    deleteMutation.mutate(selectedCategory.id);
  };

  const CustomTreeItem = React.forwardRef(function CustomTreeItem(
    props: TreeItemProps,
    ref: React.Ref<HTMLLIElement>,
  ) {
    const { itemId, label, ...other } = props;
    const categoryId = parseInt(itemId, 10);
    const category = categories?.data.find((c) => c.id === categoryId);

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

  return (
    <PageLayout title="Категории">
      <Button variant="contained" onClick={() => openDialog()} sx={{ mb: 2 }}>
        Добавить категорию
      </Button>
      {isPending && (
        <div style={{ display: "grid", gap: "0.5rem" }}>
          {range(5).map((key) => (
            <Skeleton key={key} variant="rounded" width="100%" height={50} />
          ))}
        </div>
      )}
      {!isPending && (
        <RichTreeView
          items={categoriesTree}
          className={classes.tree}
          itemChildrenIndentation={24}
          slots={{ item: CustomTreeItem }}
        />
      )}
      {isDialogOpen && (
        <CategoryForm
          open={isDialogOpen}
          onClose={() => {
            setIsDialogOpen(false);
            setSelectedCategory(null);
          }}
          categories={categories?.data || []}
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
