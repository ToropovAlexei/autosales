import { InputAutocomplete } from "@/components";
import { ENDPOINTS } from "@/constants";
import { useOne } from "@/hooks";
import { dataLayer } from "@/lib/dataLayer";
import { BotSettings, UpdateBotSettings } from "@/types";
import { queryKeys } from "@/utils/query";
import { Button, Stack } from "@mui/material";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { FormProvider, useForm } from "react-hook-form";
import { toast } from "react-toastify";

export const InputSupportOperators = () => {
  const { data: settings, isPending: isSettingsPending } = useOne<BotSettings>({
    endpoint: ENDPOINTS.BOT_SETTINGS,
  });

  const queryClient = useQueryClient();
  const supportOperatorsOptions =
    settings?.bot_payment_system_support_operators ?? [];

  const { mutate, isPending } = useMutation({
    mutationFn: async (params: UpdateBotSettings) =>
      dataLayer.update({
        url: ENDPOINTS.BOT_SETTINGS,
        params,
      }),
    onSuccess: () => {
      toast.success("Настройки бота сохранены");
      queryClient.invalidateQueries({
        queryKey: queryKeys.one(ENDPOINTS.BOT_SETTINGS),
      });
    },
    onError: () => toast.error("Ошибка сохранения настроек"),
  });

  const form = useForm<UpdateBotSettings>({ defaultValues: settings });
  const { handleSubmit, reset, formState, setValue, watch } = form;

  useEffect(() => {
    if (settings) {
      reset(settings);
    }
  }, [settings, reset]);

  const onSubmit = (data: UpdateBotSettings) => {
    mutate(data);
  };

  return (
    <>
      <FormProvider {...form}>
        <Stack gap={2} component="form" onSubmit={handleSubmit(onSubmit)}>
          <InputAutocomplete
            name="bot_payment_system_support_operators"
            label="Операторы поддержки платежной системы"
            options={supportOperatorsOptions}
            multiple
            freeSolo
            placeholder="Введите логины операторов и нажмите Enter, максимум 3 оператора"
            disabled={isSettingsPending}
          />
          <Button
            type="submit"
            variant="contained"
            disabled={!formState.isDirty || isPending || isSettingsPending}
            sx={{ width: "fit-content" }}
          >
            Сохранить
          </Button>
        </Stack>
      </FormProvider>
    </>
  );
};
