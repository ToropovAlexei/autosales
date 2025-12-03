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
import { useList } from "@/hooks";

interface PaymentGateway {
  name: string;
  display_name: string;
}

interface PaymentMarkupsFormData {
  [key: string]: number;
}

interface PaymentMarkupsFormProps {
  settings: { [key: string]: string } | undefined;
  isSettingsPending: boolean;
  refetchSettings: () => void;
}

export const PaymentMarkupsForm = ({
  settings,
  isSettingsPending,
  refetchSettings,
}: PaymentMarkupsFormProps) => {
  const queryClient = useQueryClient();
  const { data: gatewaysData } = useList<PaymentGateway>({
    endpoint: ENDPOINTS.ADMIN_PAYMENT_PROVIDERS,
  });
  const gateways = gatewaysData?.data || [];

  const form = useForm<PaymentMarkupsFormData>({
    defaultValues: {},
  });
  const { handleSubmit, reset, formState, setValue } = form;

  useEffect(() => {
    if (settings && gateways.length > 0) {
      const defaultValues: PaymentMarkupsFormData = {};
      gateways.forEach((gateway) => {
        const key = `GATEWAY_MARKUP_${gateway.name.toUpperCase()}`;
        defaultValues[key] = Number(settings[key] || 0);
      });
      reset(defaultValues);
    }
  }, [settings, gateways, reset]);

  const { mutate, isPending } = useMutation({
    mutationFn: async (data: PaymentMarkupsFormData) => {
      const params: { [key: string]: string } = {};
      for (const key in data) {
        params[key] = String(data[key]);
      }
      return dataLayer.update({
        url: ENDPOINTS.ADMIN_SETTINGS,
        params,
      });
    },
    onSuccess: () => {
      toast.success("Наценки платежных систем сохранены");
      queryClient.invalidateQueries({
        queryKey: queryKeys.one(ENDPOINTS.ADMIN_SETTINGS),
      });
      refetchSettings();
    },
    onError: () => toast.error("Ошибка сохранения настроек"),
  });

  const onSubmit = (data: PaymentMarkupsFormData) => {
    mutate(data);
  };

  return (
    <Card>
      <CardHeader title="Наценки платежных систем" />
      <CardContent>
        {isSettingsPending ? (
          <p>Загрузка...</p>
        ) : (
          <FormProvider {...form}>
            <Stack gap={2} component="form" onSubmit={handleSubmit(onSubmit)}>
              {gateways.map((gateway) => (
                <InputSlider
                  key={gateway.name}
                  name={`GATEWAY_MARKUP_${gateway.name.toUpperCase()}`}
                  label={`Наценка для ${gateway.display_name}`}
                  min={0}
                  max={25}
                  step={0.5}
                />
              ))}
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
