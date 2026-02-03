"use client";

import { Card, CardContent, Button, Stack, Box } from "@mui/material";
import { useForm, FormProvider } from "react-hook-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { ENDPOINTS } from "@/constants";
import { toast } from "react-toastify";
import { queryKeys } from "@/utils/query";
import { useEffect, useState } from "react";
import { InputAutocomplete, InputText, SelectImage } from "@/components";
import { CONFIG } from "../../../../../config";
import { BotSettings, UpdateBotSettings } from "@/types/settings";
import { useOne } from "@/hooks";
import { ImageResponse } from "@/types/image";

export const BotSettingsForm = () => {
  const { data: settings, isPending: isSettingsPending } = useOne<BotSettings>({
    endpoint: ENDPOINTS.BOT_SETTINGS,
  });

  const queryClient = useQueryClient();
  const [isNewUserImageSelectorOpen, setIsNewUserImageSelectorOpen] =
    useState(false);
  const [
    isReturningUserImageSelectorOpen,
    setIsReturningUserImageSelectorOpen,
  ] = useState(false);
  const [isSupportImageSelectorOpen, setIsSupportImageSelectorOpen] =
    useState(false);

  const form = useForm<UpdateBotSettings>({ defaultValues: settings });
  const { handleSubmit, reset, formState, setValue, watch } = form;

  useEffect(() => {
    if (settings) {
      reset(settings);
    }
  }, [settings, reset]);

  const newUserImageId = watch("bot_messages_new_user_welcome_image_id");
  const returningUserImageId = watch(
    "bot_messages_returning_user_welcome_image_id",
  );
  const supportImageId = watch("bot_messages_support_image_id");
  const supportOperatorsOptions =
    settings?.bot_payment_system_support_operators ?? [];

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

  const handleSelectImage = (
    field: keyof UpdateBotSettings,
    image: ImageResponse,
  ) => {
    setValue(field, image.id, { shouldDirty: true });
    setIsNewUserImageSelectorOpen(false);
    setIsReturningUserImageSelectorOpen(false);
    setIsSupportImageSelectorOpen(false);
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
                <Box>
                  <Button
                    variant="outlined"
                    onClick={() => setIsNewUserImageSelectorOpen(true)}
                  >
                    Выбрать изображение
                  </Button>
                  {newUserImageId && (
                    <img
                      src={`${CONFIG.IMAGES_URL}/${newUserImageId}`}
                      alt="Preview"
                      style={{
                        width: "100px",
                        height: "100px",
                        objectFit: "cover",
                        marginLeft: "1rem",
                      }}
                    />
                  )}
                </Box>
                <InputText
                  name="bot_messages_new_user_welcome"
                  label="Приветственное сообщение для новых пользователей (используйте {username})"
                  multiline
                  minRows={4}
                />

                <Box>
                  <Button
                    variant="outlined"
                    onClick={() => setIsReturningUserImageSelectorOpen(true)}
                  >
                    Выбрать изображение
                  </Button>
                  {returningUserImageId && (
                    <img
                      src={`${CONFIG.IMAGES_URL}/${returningUserImageId}`}
                      alt="Preview"
                      style={{
                        width: "100px",
                        height: "100px",
                        objectFit: "cover",
                        marginLeft: "1rem",
                      }}
                    />
                  )}
                </Box>
                <InputText
                  name="bot_messages_returning_user_welcome"
                  label="Приветственное сообщение для вернувшихся пользователей (используйте {username})"
                  multiline
                  minRows={4}
                />

                <Box>
                  <Button
                    variant="outlined"
                    onClick={() => setIsSupportImageSelectorOpen(true)}
                  >
                    Выбрать изображение
                  </Button>
                  {supportImageId && (
                    <img
                      src={`${CONFIG.IMAGES_URL}/${supportImageId}`}
                      alt="Preview"
                      style={{
                        width: "100px",
                        height: "100px",
                        objectFit: "cover",
                        marginLeft: "1rem",
                      }}
                    />
                  )}
                </Box>
                <InputText
                  name="bot_messages_support"
                  label="Сообщение поддержки"
                  multiline
                  minRows={2}
                />
                <InputAutocomplete
                  name="bot_payment_system_support_operators"
                  label="Операторы поддержки платежной системы"
                  options={supportOperatorsOptions}
                  multiple
                  freeSolo
                  placeholder="Введите логины операторов и нажмите Enter, максимум 3 оператора"
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

      <SelectImage
        open={isNewUserImageSelectorOpen}
        onClose={() => setIsNewUserImageSelectorOpen(false)}
        onSelect={(image) =>
          handleSelectImage("bot_messages_new_user_welcome_image_id", image)
        }
      />
      <SelectImage
        open={isReturningUserImageSelectorOpen}
        onClose={() => setIsReturningUserImageSelectorOpen(false)}
        onSelect={(image) =>
          handleSelectImage(
            "bot_messages_returning_user_welcome_image_id",
            image,
          )
        }
      />
      <SelectImage
        open={isSupportImageSelectorOpen}
        onClose={() => setIsSupportImageSelectorOpen(false)}
        onSelect={(image) =>
          handleSelectImage("bot_messages_support_image_id", image)
        }
      />
    </>
  );
};
