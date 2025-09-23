"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
  CardFooter,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { List } from "@/components/List";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import api from "@/lib/api";
import { queryKeys } from "@/utils/query";
import { dataLayer } from "@/lib/dataLayer";

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
      api.put(`/referrals/${botId}/status`, { is_active: isActive }),
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
        meta: { ":id": botId },
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

  if (isPending) return <div>Loading...</div>;

  return (
    <List title="Управление реферальными ботами">
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3 p-4">
        {referralBots?.data?.map((bot: ReferralBot) => (
          <Card key={bot.id}>
            <CardHeader>
              <CardTitle className="flex items-center justify-between">
                <span>Боt ID: {bot.id}</span>
                <div className="flex items-center gap-2">
                  {bot.is_primary && <Badge>Основной</Badge>}
                  <Badge variant={bot.is_active ? "default" : "destructive"}>
                    {bot.is_active ? "Активен" : "Неактивен"}
                  </Badge>
                </div>
              </CardTitle>
              <CardDescription>
                TG ID владельца: {bot.owner_telegram_id}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              <p className="text-sm text-muted-foreground truncate">
                Токен: ...{bot.bot_token.slice(-8)}
              </p>
              <p className="text-sm text-muted-foreground">
                Оборот: {bot.turnover.toFixed(2)} ₽
              </p>
              <p className="text-sm text-muted-foreground">
                Начисления: {bot.accruals.toFixed(2)} ₽
              </p>
              <div className="flex items-center justify-between pt-4">
                <span className="text-sm font-medium">Статус</span>
                <Switch
                  checked={bot.is_active}
                  onCheckedChange={(isActive) =>
                    updateStatusMutation.mutate({ botId: bot.id, isActive })
                  }
                  disabled={updateStatusMutation.isPending}
                />
              </div>
            </CardContent>
            <CardFooter className="flex justify-between">
              <Button
                variant="outline"
                onClick={() => setPrimaryMutation.mutate(bot.id)}
                disabled={bot.is_primary || setPrimaryMutation.isPending}
              >
                Сделать основным
              </Button>
              <AlertDialog>
                <AlertDialogTrigger asChild>
                  <Button variant="destructive">Удалить</Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>Вы уверены?</AlertDialogTitle>
                    <AlertDialogDescription>
                      Это действие невозможно отменить. Бот будет удален
                      навсегда.
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel>Отмена</AlertDialogCancel>
                    <AlertDialogAction
                      onClick={() => deleteMutation.mutate(bot.id)}
                    >
                      Удалить
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            </CardFooter>
          </Card>
        ))}
        {/* TODO: Add card for creating a new bot if count < 3 */}
      </div>
    </List>
  );
}
