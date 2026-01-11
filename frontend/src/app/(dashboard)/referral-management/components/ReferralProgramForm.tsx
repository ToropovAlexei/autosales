"use client";

import { Card, CardContent, Button, Stack } from "@mui/material";
import { useForm, FormProvider, useWatch } from "react-hook-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { ENDPOINTS } from "@/constants";
import { toast } from "react-toastify";
import { queryKeys } from "@/utils/query";
import { useEffect } from "react";
import { InputNumber, InputSwitch } from "@/components";
import { PricingSettings, UpdatePricingSettings } from "@/types/settings";
import { useOne } from "@/hooks";

export const ReferralProgramForm = () => {
  const { data: settings, isPending: isSettingsPending } =
    useOne<PricingSettings>({
      endpoint: ENDPOINTS.PRICING_SETTINGS,
    });
  const queryClient = useQueryClient();

  const form = useForm<UpdatePricingSettings>({
    defaultValues: {
      referral_program_enabled: !!settings?.referral_program_enabled,
      referral_percentage: Number(settings?.referral_percentage || 0),
    },
  });
  const { handleSubmit, reset, formState, control } = form;

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
      toast.success("Настройки реферальной программы сохранены");
      queryClient.invalidateQueries({
        queryKey: queryKeys.one(ENDPOINTS.PRICING_SETTINGS),
      });
    },
    onError: () => toast.error("Ошибка сохранения настроек"),
  });

  const onSubmit = (data: UpdatePricingSettings) => {
    mutate(data);
  };

  const referralProgramEnabled = useWatch({
    name: "referral_program_enabled",
    control,
  });

  return (
    <Card>
      <CardContent>
        {isSettingsPending ? (
          <p>Загрузка...</p>
        ) : (
          <FormProvider {...form}>
            <Stack gap={2} component="form" onSubmit={handleSubmit(onSubmit)}>
              <InputSwitch
                name="referral_program_enabled"
                label="Включить реферальную программу"
              />
              <InputNumber
                name="referral_percentage"
                label="Процент отчислений рефоводам (%)"
                disabled={!referralProgramEnabled}
                rules={{
                  min: {
                    value: 0,
                    message: "Процент не может быть меньше 0",
                  },
                  max: {
                    value: 100,
                    message: "Процент не может быть больше 100",
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
