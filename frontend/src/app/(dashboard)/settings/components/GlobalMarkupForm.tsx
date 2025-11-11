"use client";

import { Card, CardContent, CardHeader, Button, Stack } from "@mui/material";
import { useForm, FormProvider } from "react-hook-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { ENDPOINTS } from "@/constants";
import { toast } from "react-toastify";
import { queryKeys } from "@/utils/query";
import { useEffect } from "react";
import { InputSlider } from "@/components";

interface GlobalMarkupFormData {
  GLOBAL_PRICE_MARKUP: number;
}

interface GlobalMarkupFormProps {
  settings: { [key: string]: string } | undefined;
  isSettingsPending: boolean;
  refetchSettings: () => void;
}

export const GlobalMarkupForm = ({
  settings,
  isSettingsPending,
  refetchSettings,
}: GlobalMarkupFormProps) => {
  const queryClient = useQueryClient();

  const form = useForm<GlobalMarkupFormData>({
    defaultValues: {
      GLOBAL_PRICE_MARKUP: Number(settings?.GLOBAL_PRICE_MARKUP || 0),
    },
  });
  const { handleSubmit, reset, formState } = form;

  useEffect(() => {
    if (settings) {
      reset({
        GLOBAL_PRICE_MARKUP: Number(settings.GLOBAL_PRICE_MARKUP || 0),
      });
    }
  }, [settings, reset]);

  const { mutate, isPending } = useMutation({
    mutationFn: async (data: GlobalMarkupFormData) => {
      return dataLayer.update({
        url: ENDPOINTS.ADMIN_SETTINGS,
        params: {
          GLOBAL_PRICE_MARKUP: String(data.GLOBAL_PRICE_MARKUP),
        },
      });
    },
    onSuccess: () => {
      toast.success("Глобальная наценка сохранена");
      queryClient.invalidateQueries({
        queryKey: queryKeys.one(ENDPOINTS.ADMIN_SETTINGS),
      });
      refetchSettings();
    },
    onError: () => toast.error("Ошибка сохранения настроек"),
  });

  const onSubmit = (data: GlobalMarkupFormData) => {
    mutate(data);
  };

  return (
    <Card>
      <CardHeader title="Глобальная наценка на товары" />
      <CardContent>
        {isSettingsPending ? (
          <p>Загрузка...</p>
        ) : (
          <FormProvider {...form}>
            <Stack gap={2} component="form" onSubmit={handleSubmit(onSubmit)}>
              <InputSlider
                name="GLOBAL_PRICE_MARKUP"
                label="Наценка"
                min={0}
                max={30}
                step={1}
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
