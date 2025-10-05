"use client";

import { FormProvider, useForm } from "react-hook-form";
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
} from "@mui/material";
import { ICategory, IProduct } from "@/types";
import { InputText, InputSelect, InputNumber } from "@/components/form";
import { useState } from "react";
import { ConfirmModal } from "@/components";

interface ProductFormData {
  name: string;
  category_id: number;
  price: number;
  initial_stock?: number;
  type: "item" | "subscription";
  subscription_period_days?: number;
}

interface ProductFormProps {
  open: boolean;
  onClose: () => void;
  onConfirm: (data: Partial<IProduct>) => void;
  defaultValues?: IProduct;
  categories: { value: number; label: string }[];
  allCategories: ICategory[];
}

const findCategory = (
  categories: ICategory[],
  id: number
): ICategory | null => {
  for (const category of categories) {
    if (category.id === id) return category;
    if (category.sub_categories) {
      const found = findCategory(category.sub_categories, id);
      if (found) return found;
    }
  }
  return null;
};

export const ProductForm = ({
  open,
  onClose,
  onConfirm,
  defaultValues,
  categories,
  allCategories,
}: ProductFormProps) => {
  const isEditMode = !!defaultValues?.id;
  const [confirmState, setConfirmState] = useState<{
    open: boolean;
    data: ProductFormData | null;
  }>({ open: false, data: null });

  const form = useForm<ProductFormData>({
    defaultValues: {
      name: defaultValues?.name || "",
      category_id: defaultValues?.category_id || 0,
      price: defaultValues?.price || 0,
      type: defaultValues?.type || "item",
      initial_stock: !isEditMode ? 0 : undefined,
      subscription_period_days: defaultValues?.subscription_period_days || 30,
    },
  });
  const { handleSubmit, watch } = form;

  const productType = watch("type");

  const proceedToConfirm = (data: ProductFormData) => {
    const payload: Partial<IProduct> = {
      id: defaultValues?.id,
      name: data.name,
      category_id: data.category_id,
      price: data.price,
      type: data.type,
    };

    if (data.type === "item") {
      if (!isEditMode) {
        payload.initial_stock = data.initial_stock;
      }
    } else {
      payload.subscription_period_days = data.subscription_period_days;
    }

    onConfirm(payload);
  };

  const handleFormSubmit = (data: ProductFormData) => {
    const selectedCategory = findCategory(allCategories, data.category_id);

    if (
      selectedCategory &&
      selectedCategory.sub_categories &&
      selectedCategory.sub_categories.length > 0
    ) {
      setConfirmState({ open: true, data });
    } else {
      proceedToConfirm(data);
    }
  };

  const handleConfirmation = () => {
    if (confirmState.data) {
      proceedToConfirm(confirmState.data);
    }
    setConfirmState({ open: false, data: null });
  };

  return (
    <>
      <Dialog open={open} onClose={onClose} fullWidth maxWidth="sm">
        <DialogTitle>
          {isEditMode ? "Редактировать товар" : "Добавить товар"}
        </DialogTitle>
        <FormProvider {...form}>
          <form onSubmit={handleSubmit(handleFormSubmit)}>
            <DialogContent
              sx={{
                display: "flex",
                flexDirection: "column",
                gap: 2,
                pt: "8px !important",
              }}
            >
              <InputSelect
                name="type"
                label="Тип"
                options={[
                  { value: "item", label: "Товар" },
                  { value: "subscription", label: "Подписка" },
                ]}
              />
              <InputText name="name" label="Название" required />
              <InputSelect
                name="category_id"
                label="Категория"
                options={categories}
              />
              <InputNumber name="price" label="Цена" required />

              {productType === "item" && !isEditMode && (
                <InputNumber
                  name="initial_stock"
                  label="Начальный остаток"
                  required
                />
              )}

              {productType === "subscription" && (
                <InputNumber
                  name="subscription_period_days"
                  label="Срок (дней)"
                  required
                />
              )}
            </DialogContent>
            <DialogActions>
              <Button onClick={onClose}>Отмена</Button>
              <Button type="submit" variant="contained">
                {isEditMode ? "Сохранить" : "Добавить"}
              </Button>
            </DialogActions>
          </form>
        </FormProvider>
      </Dialog>

      <ConfirmModal
        open={confirmState.open}
        onClose={() => setConfirmState({ open: false, data: null })}
        onConfirm={handleConfirmation}
        title="Подтверждение"
        contentText="Вы уверены, что хотите разместить товар в категории, у которой есть подкатегории?"
        confirmBtnText="Да, уверен"
      />
    </>
  );
};
