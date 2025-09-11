"use client";

import { useOne, useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { toast } from "sonner";
import { useMutation } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { useDebouncedCallback } from "@tanstack/react-pacer";

interface User {
  id: number;
  email: string;
  is_active: boolean;
  role: string;
  referral_program_enabled: boolean;
  referral_percentage: number;
}

interface ReferralBot {
  id: number;
  owner_id: number;
  seller_id: number;
  bot_token: string; // Should be masked
  is_active: boolean;
  created_at: string;
}

export default function SettingsPage() {
  const {
    data: user,
    isPending: isUserPending,
    refetch,
  } = useOne<User>({
    endpoint: ENDPOINTS.USERS_ME,
  });

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
    onSuccess: () => {
      refetch();
      toast.success("Настройки сохранены");
    },
    onError: () => {
      toast.error("Ошибка сохранения");
    },
  });

  const debouncedMutate = useDebouncedCallback(mutate, { wait: 1000 });

  const { data: referralBots, isPending: isBotsPending } = useList<ReferralBot>(
    { endpoint: ENDPOINTS.REFERRALS }
  );

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">Настройки</h1>
      <Card className="mb-6">
        <CardHeader>
          <CardTitle>Реферальная программа</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {isUserPending ? (
            <p>Загрузка...</p>
          ) : (
            <>
              <div className="flex items-center space-x-2">
                <Switch
                  id="referral-enabled"
                  checked={!!user?.referral_program_enabled}
                  onCheckedChange={(checked) => {
                    mutate({ referral_program_enabled: checked });
                  }}
                />
                <Label htmlFor="referral-enabled">
                  Включить реферальную программу
                </Label>
              </div>
              <div className="space-y-1">
                <Label htmlFor="referral-percentage">
                  Процент отчислений рефоводам (%)
                </Label>
                <Input
                  id="referral-percentage"
                  type="number"
                  value={user?.referral_percentage || 0}
                  onChange={(e) =>
                    debouncedMutate({
                      referral_percentage: Number(e.target.value),
                    })
                  }
                  disabled={!user?.referral_program_enabled}
                  className="max-w-xs"
                />
              </div>
            </>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Реферальные боты</CardTitle>
        </CardHeader>
        <CardContent>
          {isBotsPending ? (
            <p>Загрузка...</p>
          ) : (
            <div className="space-y-2">
              {referralBots?.data?.map((bot) => (
                <div key={bot.id} className="p-2 border rounded-md">
                  <p>ID: {bot.id}</p>
                  <p>Владелец: {bot.owner_id}</p>
                  <p>
                    Дата создания:{" "}
                    {new Date(bot.created_at).toLocaleDateString()}
                  </p>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
