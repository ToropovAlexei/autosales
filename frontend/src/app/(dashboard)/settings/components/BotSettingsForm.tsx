"use client";

import { Card, CardContent, CardHeader, Button, Stack } from "@mui/material";
import { useForm, FormProvider } from "react-hook-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { ENDPOINTS } from "@/constants";
import { toast } from "react-toastify";
import { queryKeys } from "@/utils/query";
import { useEffect } from "react";
import { InputText } from "@/components";

interface BotSettingsFormData {
  new_user_welcome_message: string;
  returning_user_welcome_message: string;
  support_message: string;
}

interface BotSettingsFormProps {
  settings: { [key: string]: string } | undefined;
  isSettingsPending: boolean;
  refetchSettings: () => void;
}

export const BotSettingsForm = ({
  settings,
  isSettingsPending,
  refetchSettings,
}: BotSettingsFormProps) => {
  const queryClient = useQueryClient();

  const form = useForm<BotSettingsFormData>({
    defaultValues: {
      new_user_welcome_message: settings?.new_user_welcome_message || "",
      returning_user_welcome_message:
        settings?.returning_user_welcome_message || "",
      support_message: settings?.support_message || "",
    },
  });
  const { handleSubmit, reset, formState } = form;

  useEffect(() => {
    if (settings) {
      reset({
        new_user_welcome_message: settings.new_user_welcome_message || "",
        returning_user_welcome_message:
          settings.returning_user_welcome_message || "",
        support_message: settings.support_message || "",
      });
    }
  }, [settings, reset]);

  const { mutate, isPending } = useMutation({
    mutationFn: async (data: BotSettingsFormData) => {
      return dataLayer.update({
        url: ENDPOINTS.ADMIN_SETTINGS,
        params: {
          new_user_welcome_message: data.new_user_welcome_message,
          returning_user_welcome_message: data.returning_user_welcome_message,
          support_message: data.support_message,
        },
      });
    },
    onSuccess: () => {
      toast.success("Настройки бота сохранены");
      queryClient.invalidateQueries({
        queryKey: queryKeys.one(ENDPOINTS.ADMIN_SETTINGS),
      });
      refetchSettings();
    },
    onError: () => toast.error("Ошибка сохранения настроек"),
  });

  const onSubmit = (data: BotSettingsFormData) => {
    mutate(data);
  };

  return (
    <Card>
      <CardHeader title="Настройки бота" />
      <CardContent>
        {isSettingsPending ? (
          <p>Загрузка...</p>
        ) : (
          <FormProvider {...form}>
            <Stack gap={2} component="form" onSubmit={handleSubmit(onSubmit)}>
              <InputText
                name="new_user_welcome_message"
                label="Приветственное сообщение для новых пользователей (используйте {username})"
                multiline
                minRows={4}
              />
              <InputText
                name="returning_user_welcome_message"
                label="Приветственное сообщение для вернувшихся пользователей (используйте {username})"
                multiline
                minRows={4}
              />
              <InputText
                name="support_message"
                label="Сообщение поддержки"
                multiline
                minRows={2}
              />
              <Button
                type="submit"
                variant="contained"
                disabled={!formState.isDirty || isPending}
                sx={{ width: "fit-content" }}
              >
                Сохранить
              </Button>
            </Stack>
          </FormProvider>
        )}
      </CardContent>
    </Card>
  );
};
