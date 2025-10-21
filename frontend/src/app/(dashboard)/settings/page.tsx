"use client";

import { useOne } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import {
  Card,
  CardContent,
  CardHeader,
  TextField,
  Switch,
  FormControlLabel,
} from "@mui/material";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { useDebouncedCallback } from "@tanstack/react-pacer";
import { queryKeys } from "@/utils/query";
import { PageLayout } from "@/components/PageLayout";
import { toast } from "react-toastify";

interface Settings {
  [key: string]: string;
}

export default function SettingsPage() {
  const {
    data: settings,
    isPending: isSettingsPending,
    refetch,
  } = useOne<Settings>({
    endpoint: ENDPOINTS.ADMIN_SETTINGS,
  });
  const client = useQueryClient();

  const { mutate } = useMutation({
    mutationFn: async (opts: { [key: string]: any }) => {
      if (!settings) {
        return;
      }
      return dataLayer.update({
        url: ENDPOINTS.ADMIN_SETTINGS,
        params: {
          ...settings,
          ...opts,
        },
      });
    },
    onSuccess: () => toast.success("Настройки сохранены"),
    onError: () => toast.error("Ошибка сохранения"),
    onSettled: () => refetch(),
  });

  const debouncedMutate = useDebouncedCallback(mutate, { wait: 1000 });

  const optimisticMutation = (key: string, value: any) => {
    const queryKey = queryKeys.one(ENDPOINTS.ADMIN_SETTINGS);
    client.setQueryData(queryKey, (old: any) => ({
      ...old,
      [key]: value,
    }));
    debouncedMutate({ [key]: value });
  };

  const referralProgramEnabled = settings?.referral_program_enabled === "true";
  const referralPercentage = Number(settings?.referral_percentage || 0);

  return (
    <PageLayout title="Настройки">
      <Card sx={{ mb: 3 }}>
        <CardHeader title="Реферальная программа" />
        <CardContent>
          {isSettingsPending ? (
            <p>Загрузка...</p>
          ) : (
            <>
              <FormControlLabel
                control={
                  <Switch
                    checked={referralProgramEnabled}
                    onChange={(e) =>
                      mutate({
                        referral_program_enabled: String(e.target.checked),
                      })
                    }
                  />
                }
                label="Включить реферальную программу"
              />
              <TextField
                label="Процент отчислений рефоводам (%)"
                type="number"
                value={referralPercentage}
                onChange={(e) =>
                  optimisticMutation("referral_percentage", e.target.value)
                }
                disabled={!referralProgramEnabled}
                fullWidth
                margin="normal"
                size="small"
              />
            </>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader title="Бонусы платежных систем" />
        <CardContent>
          {isSettingsPending ? (
            <p>Загрузка...</p>
          ) : (
            <>
              <TextField
                label="Бонус для Mock Provider (%)"
                type="number"
                value={Number(settings?.GATEWAY_BONUS_mock_provider || 0)}
                onChange={(e) =>
                  optimisticMutation(
                    "GATEWAY_BONUS_mock_provider",
                    e.target.value
                  )
                }
                fullWidth
                margin="normal"
                size="small"
              />
              <TextField
                label="Бонус для Platform (Карта) (%)"
                type="number"
                value={Number(settings?.GATEWAY_BONUS_platform_card || 0)}
                onChange={(e) =>
                  optimisticMutation(
                    "GATEWAY_BONUS_platform_card",
                    e.target.value
                  )
                }
                fullWidth
                margin="normal"
                size="small"
              />
              <TextField
                label="Бонус для Platform (СБП) (%)"
                type="number"
                value={Number(settings?.GATEWAY_BONUS_platform_sbp || 0)}
                onChange={(e) =>
                  optimisticMutation(
                    "GATEWAY_BONUS_platform_sbp",
                    e.target.value
                  )
                }
                fullWidth
                margin="normal"
                size="small"
              />
            </>
          )}
        </CardContent>
      </Card>

      <Card sx={{ mb: 3 }}>
        <CardHeader title="Настройки бота" />
        <CardContent>
          {isSettingsPending ? (
            <p>Загрузка...</p>
          ) : (
            <>
              <TextField
                label="Сообщение поддержки"
                multiline
                rows={4}
                value={settings?.support_message || ""}
                onChange={(e) =>
                  optimisticMutation("support_message", e.target.value)
                }
                fullWidth
                margin="normal"
                size="small"
              />
            </>
          )}
        </CardContent>
      </Card>
    </PageLayout>
  );
}
