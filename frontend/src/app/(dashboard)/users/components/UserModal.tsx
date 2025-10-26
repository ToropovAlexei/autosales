"use client";

import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Stack,
  Typography,
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
  tfaSecret: string | null;
  tfaQrCode: string | null;
}

type CreateUserForm = {
  email: string;
  password: string;
  role_id: number;
};

export const UserModal = ({ open, onClose, onSave, tfaSecret, tfaQrCode }: UserModalProps) => {
  const form = useForm<CreateUserForm>();

  const { data: allRoles } = useList<Role>({ endpoint: ENDPOINTS.ROLES });

  const handleSave = () => {
    form.handleSubmit(onSave)();
  };

  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="xs">
      <DialogTitle>{tfaSecret ? "Two-Factor Authentication" : "Создать пользователя"}</DialogTitle>
      <DialogContent>
        {tfaSecret ? (
          <Stack gap={2} py={2} alignItems="center">
            <Typography>Scan the QR code with your authenticator app:</Typography>
            <img src={`data:image/png;base64,${tfaQrCode}`} alt="2FA QR Code" />
            <Typography>Or enter this secret manually:</Typography>
            <Typography fontFamily="monospace">{tfaSecret}</Typography>
          </Stack>
        ) : (
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
        )}
      </DialogContent>
      <DialogActions>
        {tfaSecret ? (
          <Button onClick={onClose}>Закрыть</Button>
        ) : (
          <>
            <Button onClick={onClose}>Отмена</Button>
            <Button onClick={handleSave}>Сохранить</Button>
          </>
        )}
      </DialogActions>
    </Dialog>
  );
};
