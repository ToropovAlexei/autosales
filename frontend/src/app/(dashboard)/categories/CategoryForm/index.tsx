import { InputSelect, InputText, SelectImage } from "@/components";
import { Category, NewCategory, UpdateCategory } from "@/types";
import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Stack,
} from "@mui/material";
import { useMemo, useState } from "react";
import { FormProvider, useForm } from "react-hook-form";
import { CONFIG } from "../../../../../config";
import classes from "./styles.module.css";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { ENDPOINTS } from "@/constants";
import { queryKeys } from "@/utils/query";
import { dataLayer } from "@/lib/dataLayer";
import { toast } from "react-toastify";
import { flattenCategoriesForSelect } from "../utils";
import { ImageResponse } from "@/types/image";

interface IProps {
  open: boolean;
  onClose: () => void;
  categories: Category[];
  defaultValues?: Partial<Category>;
}

export const CategoryForm = ({
  open,
  onClose,
  defaultValues,
  categories,
}: IProps) => {
  const queryClient = useQueryClient();
  const invalidateCategories = () => {
    queryClient.invalidateQueries({
      queryKey: queryKeys.list(ENDPOINTS.CATEGORIES),
    });
  };
  const { mutate: createCategory } = useMutation<Category, Error, NewCategory>({
    mutationFn: (payload) =>
      dataLayer.create({
        url: ENDPOINTS.CATEGORIES,
        params: {
          name: payload.name,
          parent_id: payload.parent_id,
          image_id: payload.image_id,
        },
      }),
    onSuccess: (data) => {
      toast.success(`Категория ${data.name} создана`);
      invalidateCategories();
      onClose();
    },
    onError: () => {
      toast.error("Произошла ошибка");
    },
  });

  const { mutate: updateCategory } = useMutation<
    Category,
    Error,
    { id: number; params: UpdateCategory }
  >({
    mutationFn: ({ id, params }) =>
      dataLayer.update({
        url: ENDPOINTS.CATEGORIES,
        params,
        id,
      }),
    onSuccess: (data) => {
      invalidateCategories();
      toast.success(`Категория ${data.name} обновлена`);
      onClose();
    },
    onError: () => {
      toast.error("Произошла ошибка");
    },
  });

  const isCreate = !defaultValues?.id;
  const [isImageSelectorOpen, setIsImageSelectorOpen] = useState(false);

  const form = useForm<UpdateCategory>({
    defaultValues,
  });

  const { watch, setValue } = form;
  const imageId = watch("image_id");

  const handleImageSelect = (image: ImageResponse) => {
    setValue("image_id", image.id);
    setIsImageSelectorOpen(false);
  };

  const flattenedCategories = useMemo(
    () => flattenCategoriesForSelect(categories),
    [categories]
  );

  return (
    <>
      <Dialog open={open} onClose={onClose} fullWidth maxWidth="xs">
        <DialogTitle>
          {isCreate ? "Создание категории" : "Редактирование категории"}
        </DialogTitle>
        <DialogContent>
          <FormProvider {...form}>
            <Stack gap={2} py={1}>
              <InputText name="name" label="Название" required />
              <InputSelect
                name="parent_id"
                label="Родительская категория"
                options={flattenedCategories}
                noneLabel="Нет (корневая категория)"
                withNone
                displayEmpty
              />
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
            </Stack>
          </FormProvider>
        </DialogContent>
        <DialogActions>
          <Button
            variant="contained"
            onClick={form.handleSubmit((form) => {
              if (isCreate && form.name) {
                createCategory({
                  name: form.name,
                  parent_id: form.parent_id,
                  image_id: form.image_id,
                });
                return;
              }
              if (!defaultValues?.id) {
                return;
              }
              updateCategory({
                id: defaultValues.id,
                params: {
                  name: form.name,
                  parent_id: form.parent_id ? Number(form.parent_id) : null,
                  image_id: form.image_id,
                  position: form.position,
                },
              });
            })}
          >
            {isCreate ? "Создать" : "Сохранить"}
          </Button>
          <Button onClick={onClose}>Закрыть</Button>
        </DialogActions>
      </Dialog>
      <SelectImage
        open={isImageSelectorOpen}
        onClose={() => setIsImageSelectorOpen(false)}
        onSelect={handleImageSelect}
      />
    </>
  );
};
