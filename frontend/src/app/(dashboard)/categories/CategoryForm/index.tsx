import { InputSelect, InputText } from "@/components";
import { ICategory } from "@/types";
import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Stack,
} from "@mui/material";
import { FormProvider, useForm } from "react-hook-form";

type Form = {
  name: string;
  parent_id: number;
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
  const form = useForm<Form>({
    defaultValues: {
      parent_id: 0,
      ...defaultValues,
    },
  });

  return (
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
  );
};
