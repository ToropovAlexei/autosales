"use client";

import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Stack,
} from "@mui/material";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { Role } from "@/types";
import { FormProvider, useForm } from "react-hook-form";
import { InputPassword, InputSelect, InputText } from "@/components";

interface UserModalProps {
  open: boolean;
  onClose: () => void;
  onSave: (data: any) => void;
}

type CreateUserForm = {
  email: string;
  password: string;
  role_id: number;
};

export const UserModal = ({ open, onClose, onSave }: UserModalProps) => {
  const form = useForm<CreateUserForm>();

  const { data: allRoles } = useList<Role>({ endpoint: ENDPOINTS.ROLES });

  const handleSave = () => {
    form.handleSubmit(onSave)();
  };

  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="xs">
      <DialogTitle>Создать пользователя</DialogTitle>
      <DialogContent>
        <FormProvider {...form}>
          <Stack gap={2} py={2}>
            <InputText name="email" label="Email" type="email" />
            <InputPassword name="password" label="Пароль" />
            <InputSelect
              name="role_id"
              label="Роль"
              options={
                allRoles?.data.map((role) => ({
                  value: role.id,
                  label: role.name,
                })) || []
              }
            />
          </Stack>
        </FormProvider>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Отмена</Button>
        <Button onClick={handleSave}>Сохранить</Button>
      </DialogActions>
    </Dialog>
  );
};
