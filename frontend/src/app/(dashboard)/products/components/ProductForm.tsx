"use client";

import { FormProvider, useForm } from "react-hook-form";
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
} from "@mui/material";
import { useState } from "react";
import {
  ConfirmModal,
  InputImage,
  InputText,
  InputSelect,
  InputNumber,
} from "@/components";
import { Category, NewProduct, Product, UpdateProduct } from "@/types";

interface ProductFormData {
  name: string;
  category_id: number;
  base_price: number;
  image_id?: string | null;
  initial_stock?: number;
  stock?: number;
  type: "item" | "subscription";
  subscription_period_days?: number;
  fulfillment_text?: string;
  fulfillment_image_id?: string | null;
}

interface ProductFormProps {
  open: boolean;
  onClose: () => void;
  onConfirm: (
    data: NewProduct | (UpdateProduct & { id: Product["id"] }),
  ) => void;
  defaultValues?: Product;
  categories: { value: number; label: string }[];
  allCategories: Category[];
}

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
      base_price: defaultValues?.base_price || 0,
      type: defaultValues?.type || "item",
      stock: defaultValues?.stock || 0,
      image_id: defaultValues?.image_id,
      initial_stock: !isEditMode ? 0 : undefined,
      subscription_period_days: defaultValues?.subscription_period_days || 30,
      fulfillment_text: defaultValues?.fulfillment_text || "",
      fulfillment_image_id: defaultValues?.fulfillment_image_id || null,
    },
  });
  const { handleSubmit, watch } = form;

  const productType = watch("type");

  const proceedToConfirm = (data: ProductFormData) => {
    const payload: Partial<Product> = {
      id: defaultValues?.id,
      name: data.name,
      category_id: Number(data.category_id),
      base_price: Number(data.base_price),
      type: data.type,
      image_id: data.image_id,
      fulfillment_image_id: data.fulfillment_image_id,
      fulfillment_text: data.fulfillment_text,
    };

    if (data.type === "item") {
      if (isEditMode) {
        payload.stock = data.stock;
      } else {
        //@ts-ignore
        payload.initial_stock = data.initial_stock;
      }
    } else {
      payload.subscription_period_days = data.subscription_period_days;
    }

    onConfirm(payload as NewProduct | (UpdateProduct & { id: Product["id"] }));
  };

  const handleFormSubmit = (data: ProductFormData) => {
    const selectedCategory = allCategories.find(
      (c) => c.id === data.category_id,
    );
    const hasSubCategories = allCategories.some(
      (c) => c.parent_id === data.category_id,
    );

    if (selectedCategory && hasSubCategories) {
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
              <InputNumber
                name="base_price"
                label="Базовая цена"
                required
                min={0}
                rules={{
                  min: {
                    value: 0,
                    message: "Цена не может быть отрицательной",
                  },
                }}
              />

              {productType === "item" && !isEditMode && (
                <InputNumber
                  name="initial_stock"
                  label="Начальный остаток"
                  required
                  min={0}
                  rules={{
                    min: {
                      value: 0,
                      message: "Остаток не может быть отрицательным",
                    },
                  }}
                />
              )}
              {productType === "item" && isEditMode && (
                <InputNumber
                  name="stock"
                  label="Текущий остаток"
                  required
                  min={0}
                  rules={{
                    min: {
                      value: 0,
                      message: "Остаток не может быть отрицательным",
                    },
                  }}
                />
              )}
              {productType === "subscription" && (
                <InputNumber
                  name="subscription_period_days"
                  label="Срок (дней)"
                  required
                />
              )}

              <InputText
                name="fulfillment_text"
                label="Текст / Ключ для выдачи"
                multiline
                rows={4}
              />
              <InputImage
                name="fulfillment_image_id"
                label="Изображение для выдачи"
                fullWidth
                folder="fulfillment"
              />
              <InputImage
                name="image_id"
                label="Изображение для товара"
                fullWidth
                folder="product"
              />
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
