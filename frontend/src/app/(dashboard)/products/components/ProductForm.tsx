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

interface ProductFormData {
  name: string;
  category_id: number;
  price: number;
  image_id?: string;
  initial_stock?: number;
  stock?: number;
  type: "item" | "subscription";
  subscription_period_days?: number;
  fulfillment_type: "none" | "text" | "image_url";
  fulfillment_content?: string;
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
  const [isImageSelectorOpen, setIsImageSelectorOpen] = useState(false);

  const form = useForm<ProductFormData>({
    defaultValues: {
      name: defaultValues?.name || "",
      category_id: defaultValues?.category_id || 0,
      price: defaultValues?.price || 0,
      type: defaultValues?.type || "item",
      stock: defaultValues?.stock || 0,
      image_id: defaultValues?.image_url ? defaultValues.image_url.split('/').pop() : undefined,
      initial_stock: !isEditMode ? 0 : undefined,
      subscription_period_days: defaultValues?.subscription_period_days || 30,
      fulfillment_type: defaultValues?.fulfillment_type || "none",
      fulfillment_content: defaultValues?.fulfillment_content || "",
    },
  });
  const { handleSubmit, watch, setValue } = form;

  const productType = watch("type");
  const fulfillmentType = watch("fulfillment_type");
  const imageId = watch("image_id");

  const handleImageSelect = (image: { ID: string }) => {
    setValue("image_id", image.ID);
    setIsImageSelectorOpen(false);
  };

  const proceedToConfirm = (data: ProductFormData) => {
    const payload: Partial<IProduct> = {
      id: defaultValues?.id,
      name: data.name,
      category_id: data.category_id,
      price: data.price,
      type: data.type,
      image_id: data.image_id,
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

    payload.fulfillment_type = data.fulfillment_type;
    if (data.fulfillment_type !== "none") {
      payload.fulfillment_content = data.fulfillment_content;
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

              <InputSelect
                name="fulfillment_type"
                label="Тип выдачи товара"
                options={[
                  { value: "none", label: "Ничего" },
                  { value: "text", label: "Текст / Ключ" },
                  { value: "image_url", label: "Ссылка на изображение" },
                ]}
              />
              {fulfillmentType !== "none" && (
                <InputText
                  name="fulfillment_content"
                  label="Содержимое для выдачи"
                  multiline
                  rows={4}
                />
              )}
              <div className={classes.selectImg}>
                <Button
                  variant="outlined"
                  onClick={() => setIsImageSelectorOpen(true)}
                >
                  Выбрать изображение
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
    </>
  );
};
