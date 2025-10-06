import { InputSelect, InputText, SelectImage } from "@/components";
import { ICategory } from "@/types";
import {
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Stack,
} from "@mui/material";
import { useState } from "react";
import { FormProvider, useForm } from "react-hook-form";
import { CONFIG } from "../../../../../config";
import classes from "./styles.module.css";

type Form = {
  name: string;
  parent_id: number;
  image_id?: string;
};

interface IProps {
  open: boolean;
  onClose: () => void;
  onConfirm: (form: Form) => void;
  categories: { label: string; value: number }[];
  defaultValues?: Partial<ICategory>;
}

export const CategoryForm = ({
  open,
  onClose,
  onConfirm,
  defaultValues,
  categories,
}: IProps) => {
  const isCreate = !defaultValues?.id;
  const [isImageSelectorOpen, setIsImageSelectorOpen] = useState(false);

  const form = useForm<Form>({
    defaultValues: {
      parent_id: 0,
      ...defaultValues,
      image_id: defaultValues?.image_id,
    },
  });

  const { watch, setValue } = form;
  const imageId = watch("image_id");

  const handleImageSelect = (image: { ID: string }) => {
    setValue("image_id", image.ID);
    setIsImageSelectorOpen(false);
  };

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
                options={categories}
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
          <Button variant="contained" onClick={form.handleSubmit(onConfirm)}>
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
