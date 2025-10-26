"use client";

import { Card, CardContent, CardHeader, Button, Stack } from "@mui/material";
import { useForm, FormProvider } from "react-hook-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { ENDPOINTS } from "@/constants";
import { toast } from "react-toastify";
import { queryKeys } from "@/utils/query";
import { useEffect } from "react";
import { InputNumber } from "@/components";

interface PaymentDiscountsFormData {
  GATEWAY_DISCOUNT_mock_provider: number;
  GATEWAY_DISCOUNT_platform_card: number;
  GATEWAY_DISCOUNT_platform_sbp: number;
}

interface PaymentDiscountsFormProps {
  settings: { [key: string]: string } | undefined;
  isSettingsPending: boolean;
  refetchSettings: () => void;
}

export const PaymentDiscountsForm = ({
  settings,
  isSettingsPending,
  refetchSettings,
}: PaymentDiscountsFormProps) => {
  const queryClient = useQueryClient();

  const form = useForm<PaymentDiscountsFormData>({
    defaultValues: {
      GATEWAY_DISCOUNT_mock_provider: Number(
        settings?.GATEWAY_DISCOUNT_mock_provider || 0
      ),
      GATEWAY_DISCOUNT_platform_card: Number(
        settings?.GATEWAY_DISCOUNT_platform_card || 0
      ),
      GATEWAY_DISCOUNT_platform_sbp: Number(
        settings?.GATEWAY_DISCOUNT_platform_sbp || 0
      ),
    },
  });
  const { handleSubmit, reset, formState } = form;

  useEffect(() => {
    if (settings) {
      reset({
        GATEWAY_DISCOUNT_mock_provider: Number(
          settings.GATEWAY_DISCOUNT_mock_provider || 0
        ),
        GATEWAY_DISCOUNT_platform_card: Number(
          settings.GATEWAY_DISCOUNT_platform_card || 0
        ),
        GATEWAY_DISCOUNT_platform_sbp: Number(
          settings.GATEWAY_DISCOUNT_platform_sbp || 0
        ),
      });
    }
  }, [settings, reset]);

  const { mutate, isPending } = useMutation({
    mutationFn: async (data: PaymentDiscountsFormData) => {
      return dataLayer.update({
        url: ENDPOINTS.ADMIN_SETTINGS,
        params: {
          GATEWAY_DISCOUNT_mock_provider: String(data.GATEWAY_DISCOUNT_mock_provider),
          GATEWAY_DISCOUNT_platform_card: String(data.GATEWAY_DISCOUNT_platform_card),
          GATEWAY_DISCOUNT_platform_sbp: String(data.GATEWAY_DISCOUNT_platform_sbp),
        },
      });
    },
    onSuccess: () => {
      toast.success("Настройки скидок платежных систем сохранены");
      queryClient.invalidateQueries({
        queryKey: queryKeys.one(ENDPOINTS.ADMIN_SETTINGS),
      });
      refetchSettings();
    },
    onError: () => toast.error("Ошибка сохранения настроек"),
  });

  const onSubmit = (data: PaymentDiscountsFormData) => {
    mutate(data);
  };

  return (
    <Card>
      <CardHeader title="Скидки платежных систем" />
      <CardContent>
        {isSettingsPending ? (
          <p>Загрузка...</p>
        ) : (
          <FormProvider {...form}>
            <Stack component="form" onSubmit={handleSubmit(onSubmit)} gap={2}>
              <InputNumber
                name="GATEWAY_DISCOUNT_mock_provider"
                label="Скидка для Mock Provider (%)"
                disabled={isPending}
              />
              <InputNumber
                name="GATEWAY_DISCOUNT_platform_card"
                label="Скидка для Platform (Карта) (%)"
                disabled={isPending}
              />
              <InputNumber
                name="GATEWAY_DISCOUNT_platform_sbp"
                label="Скидка для Platform (СБП) (%)"
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
