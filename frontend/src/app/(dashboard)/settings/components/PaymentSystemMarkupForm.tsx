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

interface PaymentSystemMarkupFormProps {
  settings: PricingSettings | undefined;
  isSettingsPending: boolean;
  refetchSettings: () => void;
}

export const PaymentSystemMarkupForm = ({
  settings,
  isSettingsPending,
  refetchSettings,
}: PaymentSystemMarkupFormProps) => {
  const queryClient = useQueryClient();

  const form = useForm<UpdatePricingSettings>({
    defaultValues: {
      pricing_gateway_markup: Number(settings?.pricing_gateway_markup || 0),
    },
  });
  const { handleSubmit, reset, formState } = form;

  useEffect(() => {
    if (settings) {
      reset({
        pricing_gateway_markup: Number(settings.pricing_gateway_markup || 0),
      });
    }
  }, [settings, reset]);

  const { mutate, isPending } = useMutation({
    mutationFn: async (data: UpdatePricingSettings) =>
      dataLayer.update({
        url: ENDPOINTS.PRICING_SETTINGS,
        params: {
          pricing_gateway_markup: Number(data.pricing_gateway_markup),
        },
      }),
    onSuccess: () => {
      toast.success("Наценка платежной системы сохранена");
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
      <CardHeader title="Наценка платежной системы" />
      <CardContent>
        {isSettingsPending ? (
          <p>Загрузка...</p>
        ) : (
          <FormProvider {...form}>
            <Stack gap={2} component="form" onSubmit={handleSubmit(onSubmit)}>
              <InputSlider
                name="pricing_gateway_markup"
                label="Наценка платежной системы"
                min={0}
                max={25}
                step={0.5}
                disabled={isPending}
              />
              <InputNumber
                name="pricing_gateway_markup"
                label="Наценка платежной системы"
                rules={{
                  min: {
                    message: "Наценка должна быть больше 0%",
                    value: 0,
                  },
                  max: {
                    message: "Наценка должна быть меньше 25%",
                    value: 25,
                  },
                }}
                disabled={isPending}
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
