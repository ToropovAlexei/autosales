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
import { PricingSettings, UpdatePricingSettings } from "@/types/settings";

interface PaymentDiscountsFormProps {
  settings: PricingSettings | undefined;
  isSettingsPending: boolean;
  refetchSettings: () => void;
}

// TODO Temporary disabled
const GATEWAYS: { name: keyof PricingSettings; display_name: string }[] = [
  // {
  //   name: "pricing_gateway_bonus_mock_provider",
  //   display_name: "Криптоплатежи (мок-провайдер)",
  // },
  {
    name: "pricing_gateway_bonus_platform_card",
    display_name: "Платформа (Карта)",
  },
  // {
  //   name: "pricing_gateway_bonus_platform_sbp",
  //   display_name: "Платформа (СБП)",
  // },
];

export const PaymentDiscountsForm = ({
  settings,
  isSettingsPending,
  refetchSettings,
}: PaymentDiscountsFormProps) => {
  const queryClient = useQueryClient();

  const form = useForm<UpdatePricingSettings>({
    defaultValues: settings,
  });
  const { handleSubmit, reset, formState } = form;

  useEffect(() => {
    if (settings) {
      reset(settings);
    }
  }, [settings, reset]);

  const { mutate, isPending } = useMutation({
    mutationFn: async (params: UpdatePricingSettings) =>
      dataLayer.update({
        url: ENDPOINTS.PRICING_SETTINGS,
        params,
      }),
    onSuccess: () => {
      toast.success("Настройки скидок платежных систем сохранены");
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
      <CardHeader title="Бонусы при пополнении" />
      <CardContent>
        {isSettingsPending ? (
          <p>Загрузка...</p>
        ) : (
          <FormProvider {...form}>
            <Stack component="form" onSubmit={handleSubmit(onSubmit)} gap={2}>
              {GATEWAYS.map((gateway) => (
                <Fragment key={gateway.name}>
                  <InputSlider
                    key={gateway.name}
                    name={gateway.name}
                    label={`Бонус для ${gateway.display_name}`}
                    min={0}
                    max={25.8}
                    step={0.1}
                    disabled={isPending}
                  />
                  <InputNumber
                    name={gateway.name}
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
