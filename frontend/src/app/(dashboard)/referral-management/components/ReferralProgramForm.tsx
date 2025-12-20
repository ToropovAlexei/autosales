"use client";

import { Card, CardContent, CardHeader, Button, Stack } from "@mui/material";
import { useForm, FormProvider, useWatch } from "react-hook-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { ENDPOINTS } from "@/constants";
import { toast } from "react-toastify";
import { queryKeys } from "@/utils/query";
import { useEffect } from "react";
import { InputNumber, InputSwitch } from "@/components";

interface ReferralProgramFormData {
  referral_program_enabled: boolean;
  referral_percentage: number;
}

interface ReferralProgramFormProps {
  settings: { [key: string]: string } | undefined;
  isSettingsPending: boolean;
  refetchSettings: () => void;
}

export const ReferralProgramForm = ({
  settings,
  isSettingsPending,
  refetchSettings,
}: ReferralProgramFormProps) => {
  const queryClient = useQueryClient();

  const form = useForm<ReferralProgramFormData>({
    defaultValues: {
      referral_program_enabled: settings?.referral_program_enabled === "true",
      referral_percentage: Number(settings?.referral_percentage || 0),
    },
  });
  const { handleSubmit, reset, formState, control } = form;

  useEffect(() => {
    if (settings) {
      reset({
        referral_program_enabled: settings.referral_program_enabled === "true",
        referral_percentage: Number(settings.referral_percentage || 0),
      });
    }
  }, [settings, reset]);

  const { mutate, isPending } = useMutation({
    mutationFn: async (data: ReferralProgramFormData) => {
      return dataLayer.update({
        url: ENDPOINTS.ADMIN_SETTINGS,
        params: {
          referral_program_enabled: String(data.referral_program_enabled),
          referral_percentage: String(data.referral_percentage),
        },
      });
    },
    onSuccess: () => {
      toast.success("Настройки реферальной программы сохранены");
      queryClient.invalidateQueries({
        queryKey: queryKeys.one(ENDPOINTS.ADMIN_SETTINGS),
      });
      refetchSettings();
    },
    onError: () => toast.error("Ошибка сохранения настроек"),
  });

  const onSubmit = (data: ReferralProgramFormData) => {
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
