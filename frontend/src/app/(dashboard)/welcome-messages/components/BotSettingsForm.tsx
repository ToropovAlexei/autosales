"use client";

import { Card, CardContent, Button, Stack } from "@mui/material";
import { useForm, FormProvider } from "react-hook-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { ENDPOINTS } from "@/constants";
import { toast } from "react-toastify";
import { queryKeys } from "@/utils/query";
import { useEffect } from "react";
import { InputImage, InputText } from "@/components";
import { BotSettings, UpdateBotSettings } from "@/types/settings";
import { useOne } from "@/hooks";

export const BotSettingsForm = () => {
  const { data: settings, isPending: isSettingsPending } = useOne<BotSettings>({
    endpoint: ENDPOINTS.BOT_SETTINGS,
  });

  const queryClient = useQueryClient();

  const form = useForm<UpdateBotSettings>({ defaultValues: settings });
  const { handleSubmit, reset, formState } = form;

  useEffect(() => {
    if (settings) {
      reset(settings);
    }
  }, [settings, reset]);

  const { mutate, isPending } = useMutation({
    mutationFn: async (params: UpdateBotSettings) =>
      dataLayer.update({
        url: ENDPOINTS.BOT_SETTINGS,
        params,
      }),
    onSuccess: () => {
      toast.success("Настройки бота сохранены");
      queryClient.invalidateQueries({
        queryKey: queryKeys.one(ENDPOINTS.BOT_SETTINGS),
      });
    },
    onError: () => toast.error("Ошибка сохранения настроек"),
  });

  const onSubmit = (data: UpdateBotSettings) => {
    mutate(data);
  };

  return (
    <>
      <Card>
        <CardContent>
          {isSettingsPending ? (
            <p>Загрузка...</p>
          ) : (
            <FormProvider {...form}>
              <Stack gap={2} component="form" onSubmit={handleSubmit(onSubmit)}>
                <InputImage
                  name="bot_messages_new_user_welcome_image_id"
                  buttonLabel="Выбрать изображение"
                />
                <InputText
                  name="bot_messages_new_user_welcome"
                  label="Приветственное сообщение для новых пользователей (используйте {username})"
                  multiline
                  minRows={4}
                />

                <InputImage
                  name="bot_messages_returning_user_welcome_image_id"
                  buttonLabel="Выбрать изображение"
                />
                <InputText
                  name="bot_messages_returning_user_welcome"
                  label="Приветственное сообщение для вернувшихся пользователей (используйте {username})"
                  multiline
                  minRows={4}
                />

                <InputImage
                  name="bot_messages_support_image_id"
                  buttonLabel="Выбрать изображение"
                />
                <InputText
                  name="bot_messages_support"
                  label="Сообщение поддержки"
                  multiline
                  minRows={2}
                />
                <InputText
                  name="bot_description"
                  label="Описание бота"
                  multiline
                  minRows={2}
                />
                <InputText
                  name="bot_about"
                  label="О боте"
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
    </>
  );
};
