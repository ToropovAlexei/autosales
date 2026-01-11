"use client";

import { Card, CardContent, CardHeader, Button, Stack } from "@mui/material";
import { useForm, FormProvider } from "react-hook-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { ENDPOINTS } from "@/constants";
import { toast } from "react-toastify";
import { queryKeys } from "@/utils/query";
import { useEffect } from "react";
import { InputNumber, InputSlider } from "@/components";
import { PricingSettings, UpdatePricingSettings } from "@/types/settings";

interface GlobalMarkupFormProps {
  settings: PricingSettings | undefined;
  isSettingsPending: boolean;
  refetchSettings: () => void;
}

export const GlobalMarkupForm = ({
  settings,
  isSettingsPending,
  refetchSettings,
}: GlobalMarkupFormProps) => {
  const queryClient = useQueryClient();

  const form = useForm<UpdatePricingSettings>({
    defaultValues: {
      pricing_global_markup: Number(settings?.pricing_global_markup || 0),
    },
  });
  const { handleSubmit, reset, formState } = form;

  useEffect(() => {
    if (settings) {
      reset({
        pricing_global_markup: Number(settings.pricing_global_markup || 0),
      });
    }
  }, [settings, reset]);

  const { mutate, isPending } = useMutation({
    mutationFn: async (params: UpdatePricingSettings) =>
      dataLayer.update({
        url: ENDPOINTS.PRICING_SETTINGS,
        params,
      }),
    onSuccess: () => {
      toast.success("Глобальная наценка сохранена");
      queryClient.invalidateQueries({
        queryKey: queryKeys.one(ENDPOINTS.PRICING_SETTINGS),
      });
      refetchSettings();
    },
    onError: () => toast.error("Ошибка сохранения настроек"),
  });

  const onSubmit = (data: UpdatePricingSettings) => {
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
                name="pricing_global_markup"
                label="Наценка"
                min={0}
                max={1000}
                step={1}
                disabled={isPending}
              />
              <InputNumber
                name="pricing_global_markup"
                label="Наценка"
                disabled={isPending}
                rules={{
                  min: {
                    value: 0,
                    message: "Наценка не может быть отрицательной",
                  },
                  max: {
                    value: 1000,
                    message: "Наценка не может быть больше 1000%",
                  },
                }}
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
