"use client";

import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Typography,
} from "@mui/material";
import { useForm, FormProvider } from "react-hook-form";
import { InputText } from "@/components";
import { NewBot } from "@/types";

interface BotFormModalProps {
  open: boolean;
  onClose: () => void;
  onConfirm: (data: NewBot) => void;
  isCreating: boolean;
}

export const BotFormModal = ({
  open,
  onClose,
  onConfirm,
  isCreating,
}: BotFormModalProps) => {
  const form = useForm<NewBot>();
  const { handleSubmit } = form;

  return (
    <Dialog open={open} onClose={onClose} fullWidth>
      <DialogTitle>Добавить нового бота</DialogTitle>
      <FormProvider {...form}>
        <form onSubmit={handleSubmit(onConfirm)}>
          <DialogContent
            sx={{ display: "flex", flexDirection: "column", gap: 2 }}
          >
            <Typography variant="body2" color="text.secondary">
              Чтобы получить токен, откройте Telegram и найдите бота BotFather.
              Напишите ему команду <strong>/newbot</strong>, задайте имя и
              уникальный username (обычно заканчивается на <strong>_bot</strong>
              ).
            </Typography>
            <Typography variant="body2" color="text.secondary">
              BotFather пришлёт токен в ответном сообщении. Скопируйте его и
              вставьте в поле ниже. Токен — это секретный ключ, не передавайте
              его третьим лицам.
            </Typography>
            <InputText
              name="token"
              label="Токен"
              rules={{
                required: "Поле обязательно к заполнению",
                minLength: {
                  value: 44,
                  message: "Токен должен состоять минимум из 44 символов",
                },
                maxLength: {
                  value: 48,
                  message: "Токен должен состоять максимум из 48 символов",
                },
              }}
            />
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
