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

interface PaymentSystemMarkupFormData {
  PAYMENT_SYSTEM_MARKUP: number;
}

interface PaymentSystemMarkupFormProps {
  settings: { [key: string]: string } | undefined;
  isSettingsPending: boolean;
  refetchSettings: () => void;
}

export const PaymentSystemMarkupForm = ({
  settings,
  isSettingsPending,
  refetchSettings,
}: PaymentSystemMarkupFormProps) => {
  const queryClient = useQueryClient();

  const form = useForm<PaymentSystemMarkupFormData>({
    defaultValues: {
      PAYMENT_SYSTEM_MARKUP: Number(settings?.PAYMENT_SYSTEM_MARKUP || 0),
    },
  });
  const { handleSubmit, reset, formState } = form;

  useEffect(() => {
    if (settings) {
      reset({
        PAYMENT_SYSTEM_MARKUP: Number(settings.PAYMENT_SYSTEM_MARKUP || 0),
      });
    }
  }, [settings, reset]);

  const { mutate, isPending } = useMutation({
    mutationFn: async (data: PaymentSystemMarkupFormData) => {
      return dataLayer.update({
        url: ENDPOINTS.ADMIN_SETTINGS,
        params: {
          PAYMENT_SYSTEM_MARKUP: String(data.PAYMENT_SYSTEM_MARKUP),
        },
      });
    },
    onSuccess: () => {
      toast.success("Наценка платежной системы сохранена");
      queryClient.invalidateQueries({
        queryKey: queryKeys.one(ENDPOINTS.ADMIN_SETTINGS),
      });
      refetchSettings();
    },
    onError: () => toast.error("Ошибка сохранения настроек"),
  });

  const onSubmit = (data: PaymentSystemMarkupFormData) => {
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
                name="PAYMENT_SYSTEM_MARKUP"
                label="Наценка платежной системы"
                min={0}
                max={25}
                step={0.5}
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
