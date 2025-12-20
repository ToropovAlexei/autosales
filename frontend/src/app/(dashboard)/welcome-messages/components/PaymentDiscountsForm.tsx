"use client";

import { Card, CardContent, CardHeader, Button, Stack } from "@mui/material";
import { useForm, FormProvider } from "react-hook-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { ENDPOINTS } from "@/constants";
import { toast } from "react-toastify";
import { queryKeys } from "@/utils/query";
import { Fragment, useEffect } from "react";
import { InputNumber, InputSlider } from "@/components";
import { useList } from "@/hooks";

interface PaymentGateway {
  name: string;
  display_name: string;
}

interface PaymentDiscountsFormData {
  [key: string]: number;
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
  const { data: gatewaysData } = useList<PaymentGateway>({
    endpoint: ENDPOINTS.ADMIN_PAYMENT_PROVIDERS,
  });
  const gateways = gatewaysData?.data || [];

  const form = useForm<PaymentDiscountsFormData>({
    defaultValues: {},
  });
  const { handleSubmit, reset, formState } = form;

  useEffect(() => {
    if (settings && gateways.length > 0) {
      const defaultValues: PaymentDiscountsFormData = {};
      gateways.forEach((gateway) => {
        const key = `GATEWAY_BONUS_${gateway.name}`;
        defaultValues[key] = Number(settings[key] || 0);
      });
      reset(defaultValues);
    }
  }, [settings, gateways, reset]);

  const { mutate, isPending } = useMutation({
    mutationFn: async (data: PaymentDiscountsFormData) => {
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
      <CardHeader title="Бонусы при пополнении" />
      <CardContent>
        {isSettingsPending ? (
          <p>Загрузка...</p>
        ) : (
          <FormProvider {...form}>
            <Stack component="form" onSubmit={handleSubmit(onSubmit)} gap={2}>
              {gateways.map((gateway) => (
                <Fragment key={gateway.name}>
                  <InputSlider
                    key={gateway.name}
                    name={`GATEWAY_BONUS_${gateway.name}`}
                    label={`Бонус для ${gateway.display_name}`}
                    min={0}
                    max={25.8}
                    step={0.1}
                    disabled={isPending}
                  />
                  <InputNumber
                    name={`GATEWAY_BONUS_${gateway.name}`}
                    label={`Бонус для ${gateway.display_name}`}
                    rules={{
                      min: {
                        message: "Бонус не может быть отрицательным",
                        value: 0,
                      },
                      max: {
                        message: "Бонус не может быть больше 25.8%",
                        value: 25.8,
                      },
                    }}
                    disabled={isPending}
                  />
                </Fragment>
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
