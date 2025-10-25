"use client";

import { useOne } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PageLayout } from "@/components/PageLayout";
import { ReferralProgramForm } from "./components/ReferralProgramForm";
import { PaymentBonusesForm } from "./components/PaymentBonusesForm";
import { BotSettingsForm } from "./components/BotSettingsForm";
import { Stack } from "@mui/material";

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

  return (
    <PageLayout title="Настройки">
      <Stack gap={2}>
        <ReferralProgramForm
          settings={settings}
          isSettingsPending={isSettingsPending}
          refetchSettings={refetch}
        />
        <PaymentBonusesForm
          settings={settings}
          isSettingsPending={isSettingsPending}
          refetchSettings={refetch}
        />
        <BotSettingsForm
          settings={settings}
          isSettingsPending={isSettingsPending}
          refetchSettings={refetch}
        />
      </Stack>
    </PageLayout>
  );
}
