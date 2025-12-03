"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { ReferralBotCard } from "./components/ReferralBotCard";
import classes from "./styles.module.css";
import { PageLayout } from "@/components/PageLayout";

interface ReferralBot {
  id: number;
  owner_telegram_id: number;
  token: string;
  username: string;
  created_at: string;
  is_active: boolean;
  is_primary: boolean;
  turnover: number;
  accruals: number;
  referral_percentage: number;
}

export default function ReferralBotsPage() {
  const queryClient = useQueryClient();

  const { data: referralBots } = useList<ReferralBot>({
    endpoint: ENDPOINTS.ADMIN_REFERRAL_BOTS,
  });

  const updateStatusMutation = useMutation({
    mutationFn: ({ botId, isActive }: { botId: number; isActive: boolean }) =>
      dataLayer.update({
        url: `${ENDPOINTS.ADMIN_REFERRAL_BOTS}/${botId}/status`,
        params: { is_active: isActive },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ADMIN_REFERRAL_BOTS),
      });
    },
  });

  const updatePercentageMutation = useMutation({
    mutationFn: ({
      botId,
      percentage,
    }: {
      botId: number;
      percentage: number;
    }) =>
      dataLayer.update({
        url: `${ENDPOINTS.ADMIN_REFERRAL_BOTS}/${botId}/percentage`,
        params: { percentage },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ADMIN_REFERRAL_BOTS),
      });
    },
  });

  const setPrimaryMutation = useMutation({
    mutationFn: (botId: number) =>
      dataLayer.update({
        url: `${ENDPOINTS.ADMIN_REFERRAL_BOTS}/${botId}/set-primary`,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ADMIN_REFERRAL_BOTS),
      });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (botId: number) =>
      dataLayer.delete({ url: ENDPOINTS.ADMIN_REFERRAL_BOTS, id: botId }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ADMIN_REFERRAL_BOTS),
      });
    },
  });

  return (
    <PageLayout title="Управление реферальными ботами">
      <div className={classes.grid}>
        {referralBots?.data?.map((bot: ReferralBot) => (
          <ReferralBotCard
            key={bot.id}
            bot={bot}
            onUpdateStatus={updateStatusMutation.mutate}
            onSetPrimary={setPrimaryMutation.mutate}
            onUpdatePercentage={updatePercentageMutation.mutate}
            onDelete={deleteMutation.mutate}
            isUpdatingStatus={updateStatusMutation.isPending}
            isSettingPrimary={setPrimaryMutation.isPending}
          />
        ))}
      </div>
    </PageLayout>
  );
}
