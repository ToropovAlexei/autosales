"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { List } from "@/components/List";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { ReferralBotCard } from "./components/ReferralBotCard";
import classes from "./styles.module.css";
import { range } from "@/utils/array";
import { Skeleton } from "@mui/material";

interface ReferralBot {
  id: number;
  owner_telegram_id: number;
  bot_token: string;
  created_at: string;
  is_active: boolean;
  is_primary: boolean;
  turnover: number;
  accruals: number;
}

export default function ReferralBotsPage() {
  const queryClient = useQueryClient();

  const { data: referralBots, isPending } = useList<ReferralBot>({
    endpoint: ENDPOINTS.REFERRAL_BOTS_ADMIN,
  });

  const updateStatusMutation = useMutation({
    mutationFn: ({ botId, isActive }: { botId: number; isActive: boolean }) =>
      dataLayer.update({
        url: `/referrals/${botId}/status`,
        params: { is_active: isActive },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.REFERRAL_BOTS_ADMIN),
      });
    },
  });

  const setPrimaryMutation = useMutation({
    mutationFn: (botId: number) =>
      dataLayer.update({
        url: ENDPOINTS.SET_BOT_PRIMARY,
        meta: { ":id": botId.toString() },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.REFERRAL_BOTS_ADMIN),
      });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (botId: number) =>
      dataLayer.delete({ url: ENDPOINTS.REFERRALS, id: botId }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.REFERRAL_BOTS_ADMIN),
      });
    },
  });

  return (
    <List title="Управление реферальными ботами">
      <div className={classes.grid}>
        {isPending ? (
          <>
            {range(3).map((key) => (
              <Skeleton key={key} width={375} height={280} variant="rounded" />
            ))}
          </>
        ) : (
          referralBots?.data?.map((bot: ReferralBot) => (
            <ReferralBotCard
              key={bot.id}
              bot={bot}
              onUpdateStatus={updateStatusMutation.mutate}
              onSetPrimary={setPrimaryMutation.mutate}
              onDelete={deleteMutation.mutate}
              isUpdatingStatus={updateStatusMutation.isPending}
              isSettingPrimary={setPrimaryMutation.isPending}
            />
          ))
        )}
      </div>
    </List>
  );
}
