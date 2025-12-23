"use client";

import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
} from "@mui/material";
import { useForm, FormProvider } from "react-hook-form";
import { InputText } from "@/components";

interface BotFormData {
  token: string;
}

interface BotFormModalProps {
  open: boolean;
  onClose: () => void;
  onConfirm: (data: BotFormData) => void;
  isCreating: boolean;
}

export const BotFormModal = ({
  open,
  onClose,
  onConfirm,
  isCreating,
}: BotFormModalProps) => {
  const form = useForm<BotFormData>();
  const { handleSubmit } = form;

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogTitle>Добавить нового бота</DialogTitle>
      <FormProvider {...form}>
        <form onSubmit={handleSubmit(onConfirm)}>
          <DialogContent
            sx={{ display: "flex", flexDirection: "column", gap: 2 }}
          >
            <InputText name="token" label="Токен" required />
          </DialogContent>
          <DialogActions>
            <Button onClick={onClose}>Отмена</Button>
            <Button type="submit" disabled={isCreating} variant="contained">
              {isCreating ? "Создание..." : "Создать"}
            </Button>
          </DialogActions>
        </form>
      </FormProvider>
    </Dialog>
  );
};
