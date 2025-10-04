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

interface User {
  id: number;
  email: string;
  is_active: boolean;
  role: string;
  referral_program_enabled: boolean;
  referral_percentage: number;
}

export default function SettingsPage() {
  const {
    data: user,
    isPending: isUserPending,
    refetch,
  } = useOne<User>({
    endpoint: ENDPOINTS.USERS_ME,
  });
  const client = useQueryClient();

  const { mutate } = useMutation({
    mutationFn: async (opts: {
      referral_program_enabled?: boolean;
      referral_percentage?: number;
    }) => {
      if (!user) {
        return;
      }
      return dataLayer.update({
        url: ENDPOINTS.USERS_ME_REFERRAL_SETTINGS,
        params: {
          referral_program_enabled: user.referral_program_enabled,
          referral_percentage: user.referral_percentage,
          ...opts,
        },
      });
    },
    onSuccess: () => toast.success("Настройки сохранены"),
    onError: () => toast.error("Ошибка сохранения"),
    onSettled: () => refetch(),
  });

  const debouncedMutate = useDebouncedCallback(mutate, { wait: 1000 });

  const optimisticMutation = (percentage: number) => {
    const key = queryKeys.one(ENDPOINTS.USERS_ME);
    client.setQueryData(key, {
      ...client.getQueryData(key),
      referral_percentage: percentage,
    });
    debouncedMutate({ referral_percentage: percentage });
  };

  return (
    <PageLayout title="Настройки">
      <Card sx={{ mb: 3 }}>
        <CardHeader title="Реферальная программа" />
        <CardContent>
          {isUserPending ? (
            <p>Загрузка...</p>
          ) : (
            <>
              <FormControlLabel
                control={
                  <Switch
                    checked={!!user?.referral_program_enabled}
                    onChange={(e) =>
                      mutate({ referral_program_enabled: e.target.checked })
                    }
                  />
                }
                label="Включить реферальную программу"
              />
              <TextField
                label="Процент отчислений рефоводам (%)"
                type="number"
                value={user?.referral_percentage || 0}
                onChange={(e) => optimisticMutation(Number(e.target.value))}
                disabled={!user?.referral_program_enabled}
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
