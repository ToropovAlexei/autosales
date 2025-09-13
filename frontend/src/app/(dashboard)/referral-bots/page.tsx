"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Switch } from "@/components/ui/switch";
import { List } from "@/components/List";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";

interface ReferralBot {
  id: number;
  owner_telegram_id: number;
  created_at: string;
  is_active: boolean;
  turnover: number;
  accruals: number;
}

export default function ReferralBotsPage() {
  const queryClient = useQueryClient();

  const { data: referralBots, isPending } = useList<ReferralBot>({
    endpoint: ENDPOINTS.REFERRAL_BOTS_ADMIN,
  });

  const toggleMutation = useMutation({
    mutationFn: (botId: number) =>
      dataLayer.update({ url: ENDPOINTS.REFERRALS, id: botId }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.REFERRAL_BOTS_ADMIN),
      });
    },
  });

  if (isPending) return <div>Loading...</div>;

  return (
    <List title="Реферальные боты">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>ID</TableHead>
            <TableHead>Владелец (TG ID)</TableHead>
            <TableHead>Дата создания</TableHead>
            <TableHead>Оборот</TableHead>
            <TableHead>Начисления</TableHead>
            <TableHead>Статус</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {referralBots?.data?.map((bot: ReferralBot) => (
            <TableRow key={bot.id}>
              <TableCell>{bot.id}</TableCell>
              <TableCell>{bot.owner_telegram_id}</TableCell>
              <TableCell>
                {new Date(bot.created_at).toLocaleDateString()}
              </TableCell>
              <TableCell>{bot.turnover.toFixed(2)} ₽</TableCell>
              <TableCell>{bot.accruals.toFixed(2)} ₽</TableCell>
              <TableCell>
                <Switch
                  checked={bot.is_active}
                  onCheckedChange={() => toggleMutation.mutate(bot.id)}
                  disabled={toggleMutation.isPending}
                />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </List>
  );
}
