"use client";

import { useOne } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PageLayout } from "@/components/PageLayout";
import { ReferralProgramForm } from "./components/ReferralProgramForm";
import { PaymentDiscountsForm } from "./components/PaymentDiscountsForm";
import { BotSettingsForm } from "./components/BotSettingsForm";
import { Stack } from "@mui/material";
import { GlobalMarkupForm } from "./components/GlobalMarkupForm";
import { PaymentSystemMarkupForm } from "./components/PaymentSystemMarkupForm";

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
        <GlobalMarkupForm
          settings={settings}
          isSettingsPending={isSettingsPending}
          refetchSettings={refetch}
        />
        <PaymentSystemMarkupForm
          settings={settings}
          isSettingsPending={isSettingsPending}
          refetchSettings={refetch}
        />
        <PaymentDiscountsForm
          settings={settings}
          isSettingsPending={isSettingsPending}
          refetchSettings={refetch}
        />
        <ReferralProgramForm
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
