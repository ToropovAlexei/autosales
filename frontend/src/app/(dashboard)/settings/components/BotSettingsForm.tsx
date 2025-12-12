"use client";

import {
  Card,
  CardContent,
  CardHeader,
  Button,
  Stack,
  Box,
} from "@mui/material";
import { useForm, FormProvider } from "react-hook-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { ENDPOINTS } from "@/constants";
import { toast } from "react-toastify";
import { queryKeys } from "@/utils/query";
import { useEffect, useState } from "react";
import { InputText, SelectImage } from "@/components";
import { IImage } from "@/types";
import { CONFIG } from "../../../../../config";

interface BotSettingsFormData {
  new_user_welcome_message: string;
  new_user_welcome_message_image_id: string;
  returning_user_welcome_message: string;
  returning_user_welcome_message_image_id: string;
  support_message: string;
  support_message_image_id: string;
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
  const [isNewUserImageSelectorOpen, setIsNewUserImageSelectorOpen] =
    useState(false);
  const [
    isReturningUserImageSelectorOpen,
    setIsReturningUserImageSelectorOpen,
  ] = useState(false);
  const [isSupportImageSelectorOpen, setIsSupportImageSelectorOpen] =
    useState(false);

  const form = useForm<BotSettingsFormData>({
    defaultValues: {
      new_user_welcome_message: settings?.new_user_welcome_message || "",
      new_user_welcome_message_image_id:
        settings?.new_user_welcome_message_image_id || "",
      returning_user_welcome_message:
        settings?.returning_user_welcome_message || "",
      returning_user_welcome_message_image_id:
        settings?.returning_user_welcome_message_image_id || "",
      support_message: settings?.support_message || "",
      support_message_image_id: settings?.support_message_image_id || "",
    },
  });
  const { handleSubmit, reset, formState, setValue, watch } = form;

  useEffect(() => {
    if (settings) {
      reset({
        new_user_welcome_message: settings.new_user_welcome_message || "",
        new_user_welcome_message_image_id:
          settings.new_user_welcome_message_image_id || "",
        returning_user_welcome_message:
          settings.returning_user_welcome_message || "",
        returning_user_welcome_message_image_id:
          settings.returning_user_welcome_message_image_id || "",
        support_message: settings.support_message || "",
        support_message_image_id: settings.support_message_image_id || "",
      });
    }
  }, [settings, reset]);

  const newUserImageId = watch("new_user_welcome_message_image_id");
  const returningUserImageId = watch("returning_user_welcome_message_image_id");
  const supportImageId = watch("support_message_image_id");

  const { mutate, isPending } = useMutation({
    mutationFn: async (data: BotSettingsFormData) => {
      return dataLayer.update({
        url: ENDPOINTS.ADMIN_SETTINGS,
        params: {
          new_user_welcome_message: data.new_user_welcome_message,
          new_user_welcome_message_image_id:
            data.new_user_welcome_message_image_id,
          returning_user_welcome_message: data.returning_user_welcome_message,
          returning_user_welcome_message_image_id:
            data.returning_user_welcome_message_image_id,
          support_message: data.support_message,
          support_message_image_id: data.support_message_image_id,
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

  const handleSelectImage = (
    field: keyof BotSettingsFormData,
    image: IImage
  ) => {
    setValue(field, image.ID, { shouldDirty: true });
    setIsNewUserImageSelectorOpen(false);
    setIsReturningUserImageSelectorOpen(false);
    setIsSupportImageSelectorOpen(false);
  };

  return (
    <>
      <Card>
        <CardHeader title="Настройки бота" />
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
                  name="new_user_welcome_message"
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
                  name="returning_user_welcome_message"
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

      <SelectImage
        open={isNewUserImageSelectorOpen}
        onClose={() => setIsNewUserImageSelectorOpen(false)}
        onSelect={(image) =>
          handleSelectImage("new_user_welcome_message_image_id", image)
        }
      />
      <SelectImage
        open={isReturningUserImageSelectorOpen}
        onClose={() => setIsReturningUserImageSelectorOpen(false)}
        onSelect={(image) =>
          handleSelectImage("returning_user_welcome_message_image_id", image)
        }
      />
      <SelectImage
        open={isSupportImageSelectorOpen}
        onClose={() => setIsSupportImageSelectorOpen(false)}
        onSelect={(image) =>
          handleSelectImage("support_message_image_id", image)
        }
      />
    </>
  );
};
