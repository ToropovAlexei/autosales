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

interface CommissionSettingsFormData {
  AUTOSALE_COMMISSION_PERCENT: number;
  CRYPTO_PROVIDER_COMMISSION_PERCENT: number;
  PAYMENT_PROVIDER_COMMISSION_PERCENT: number;
  PLATFORM_ONLY_COMMISSION_PERCENT: number;
}

interface CommissionSettingsFormProps {
  settings: { [key: string]: string } | undefined;
  isSettingsPending: boolean;
  refetchSettings: () => void;
}

export const CommissionSettingsForm = ({
  settings,
  isSettingsPending,
  refetchSettings,
}: CommissionSettingsFormProps) => {
  const queryClient = useQueryClient();

  const form = useForm<CommissionSettingsFormData>({
    defaultValues: {
      AUTOSALE_COMMISSION_PERCENT: Number(settings?.AUTOSALE_COMMISSION_PERCENT || 0),
      CRYPTO_PROVIDER_COMMISSION_PERCENT: Number(settings?.CRYPTO_PROVIDER_COMMISSION_PERCENT || 0),
      PAYMENT_PROVIDER_COMMISSION_PERCENT: Number(settings?.PAYMENT_PROVIDER_COMMISSION_PERCENT || 0),
      PLATFORM_ONLY_COMMISSION_PERCENT: Number(settings?.PLATFORM_ONLY_COMMISSION_PERCENT || 0),
    },
  });
  const { handleSubmit, reset, formState } = form;

  useEffect(() => {
    if (settings) {
      reset({
        AUTOSALE_COMMISSION_PERCENT: Number(settings.AUTOSALE_COMMISSION_PERCENT || 0),
        CRYPTO_PROVIDER_COMMISSION_PERCENT: Number(settings.CRYPTO_PROVIDER_COMMISSION_PERCENT || 0),
        PAYMENT_PROVIDER_COMMISSION_PERCENT: Number(settings.PAYMENT_PROVIDER_COMMISSION_PERCENT || 0),
        PLATFORM_ONLY_COMMISSION_PERCENT: Number(settings.PLATFORM_ONLY_COMMISSION_PERCENT || 0),
      });
    }
  }, [settings, reset]);

  const { mutate, isPending } = useMutation({
    mutationFn: async (data: CommissionSettingsFormData) => {
      const params = Object.fromEntries(
        Object.entries(data).map(([key, value]) => [key, String(value)])
      );
      return dataLayer.update({
        url: ENDPOINTS.ADMIN_SETTINGS,
        params,
      });
    },
    onSuccess: () => {
      toast.success("Комиссии сохранены");
      queryClient.invalidateQueries({
        queryKey: queryKeys.one(ENDPOINTS.ADMIN_SETTINGS),
      });
      refetchSettings();
    },
    onError: () => toast.error("Ошибка сохранения настроек"),
  });

  const onSubmit = (data: CommissionSettingsFormData) => {
    mutate(data);
  };

  return (
    <Card>
      <CardHeader title="Комиссии" />
      <CardContent>
        {isSettingsPending ? (
          <p>Загрузка...</p>
        ) : (
          <FormProvider {...form}>
            <Stack gap={2} component="form" onSubmit={handleSubmit(onSubmit)}>
              <InputSlider
                name="AUTOSALE_COMMISSION_PERCENT"
                label="Комиссия автопродажи (%)"
                min={0}
                max={10}
                step={0.1}
              />
              <InputSlider
                name="CRYPTO_PROVIDER_COMMISSION_PERCENT"
                label="Комиссия крипто-провайдера (%)"
                min={0}
                max={10}
                step={0.05}
              />
              <InputSlider
                name="PAYMENT_PROVIDER_COMMISSION_PERCENT"
                label="Комиссия платежного провайдера (%)"
                min={0}
                max={10}
                step={0.1}
              />
              <InputSlider
                name="PLATFORM_ONLY_COMMISSION_PERCENT"
                label="Комиссия платформы (для сторонних платежек) (%)"
                min={0}
                max={10}
                step={0.1}
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
