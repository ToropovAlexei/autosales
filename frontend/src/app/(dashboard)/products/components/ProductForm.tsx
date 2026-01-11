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
  SelectImage,
  InputText,
  InputSelect,
  InputNumber,
} from "@/components";
import { CONFIG } from "../../../../../config";
import classes from "./styles.module.css";
import { ImageResponse } from "@/types/image";
import { Category, Product } from "@/types";

interface ProductFormData {
  name: string;
  category_id: number;
  base_price: number;
  image_id?: string;
  initial_stock?: number;
  stock?: number;
  type: "item" | "subscription";
  subscription_period_days?: number;
  fulfillment_text?: string;
  fulfillment_image_id?: string;
}

interface ProductFormProps {
  open: boolean;
  onClose: () => void;
  onConfirm: (data: Partial<Product>) => void;
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
  const [isImageSelectorOpen, setIsImageSelectorOpen] = useState(false);
  const [isFulfillmentImageSelectorOpen, setIsFulfillmentImageSelectorOpen] =
    useState(false);

  const form = useForm<ProductFormData>({
    defaultValues: {
      name: defaultValues?.name || "",
      category_id: defaultValues?.category_id || 0,
      base_price: defaultValues?.base_price || 0,
      type: defaultValues?.type || "item",
      stock: defaultValues?.stock || 0,
      image_id: defaultValues?.image_id || "",
      initial_stock: !isEditMode ? 0 : undefined,
      subscription_period_days: defaultValues?.subscription_period_days || 30,
      fulfillment_text: defaultValues?.fulfillment_text || "",
      fulfillment_image_id: defaultValues?.fulfillment_image_id || "",
    },
  });
  const { handleSubmit, watch, setValue } = form;

  const productType = watch("type");
  const imageId = watch("image_id");
  const fulfillmentImageId = watch("fulfillment_image_id");

  const handleImageSelect = (image: ImageResponse) => {
    setValue("image_id", image.id);
    setIsImageSelectorOpen(false);
  };

  const handleFulfillmentImageSelect = (image: ImageResponse) => {
    setValue("fulfillment_image_id", image.id);
    setIsFulfillmentImageSelectorOpen(false);
  };

  const proceedToConfirm = (data: ProductFormData) => {
    const payload: Partial<Product> = {
      id: defaultValues?.id,
      name: data.name,
      category_id: Number(data.category_id),
      base_price: Number(data.base_price),
      type: data.type,
      ...(data.image_id && { image_id: data.image_id }),
    };

    if (data.type === "item") {
      if (isEditMode) {
        payload.stock = data.stock;
      } else {
        payload.initial_stock = data.initial_stock;
      }
    } else {
      payload.subscription_period_days = data.subscription_period_days;
    }

    if (data.fulfillment_text) {
      payload.fulfillment_text = data.fulfillment_text;
    }
    if (data.fulfillment_image_id) {
      payload.fulfillment_image_id = data.fulfillment_image_id;
    }

    onConfirm(payload);
  };

  const handleFormSubmit = (data: ProductFormData) => {
    const selectedCategory = allCategories.find(
      (c) => c.id === data.category_id
    );
    const hasSubCategories = allCategories.some(
      (c) => c.parent_id === data.category_id
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
              <InputNumber name="base_price" label="Базовая цена" required />

              {productType === "item" && !isEditMode && (
                <InputNumber
                  name="initial_stock"
                  label="Начальный остаток"
                  required
                />
              )}
              {productType === "item" && isEditMode && (
                <InputNumber name="stock" label="Текущий остаток" required />
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
              <div className={classes.selectImg}>
                <Button
                  variant="outlined"
                  onClick={() => setIsFulfillmentImageSelectorOpen(true)}
                >
                  Выбрать изображение для выдачи
                </Button>
                {fulfillmentImageId && (
                  <img
                    className={classes.img}
                    src={`${CONFIG.IMAGES_URL}/${fulfillmentImageId}`}
                    alt="Fulfillment Preview"
                    width="30%"
                  />
                )}
              </div>
              <div className={classes.selectImg}>
                <Button
                  variant="outlined"
                  onClick={() => setIsImageSelectorOpen(true)}
                >
                  Выбрать изображение для товара
                </Button>
                {imageId && (
                  <img
                    className={classes.img}
                    src={`${CONFIG.IMAGES_URL}/${imageId}`}
                    alt="Preview"
                    width="30%"
                  />
                )}
              </div>
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
      <SelectImage
        open={isImageSelectorOpen}
        onClose={() => setIsImageSelectorOpen(false)}
        onSelect={handleImageSelect}
      />
      <SelectImage
        open={isFulfillmentImageSelectorOpen}
        onClose={() => setIsFulfillmentImageSelectorOpen(false)}
        onSelect={handleFulfillmentImageSelect}
      />
    </>
  );
};
