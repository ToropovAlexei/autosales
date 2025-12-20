"use client";

import { useOne } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PageLayout } from "@/components/PageLayout";
import { ReferralProgramForm } from "./components/ReferralProgramForm";
import { BotSettingsForm } from "./components/BotSettingsForm";
import { Stack } from "@mui/material";

interface Settings {
  [key: string]: string;
}

export default function WelcomeMessagesPage() {
  const {
    data: settings,
    isPending: isSettingsPending,
    refetch,
  } = useOne<Settings>({
    endpoint: ENDPOINTS.ADMIN_SETTINGS,
  });

  return (
    <PageLayout title="Приветственные сообщения">
      <Stack gap={2}>
        <BotSettingsForm
          settings={settings}
          isSettingsPending={isSettingsPending}
          refetchSettings={refetch}
        />
      </Stack>
    </PageLayout>
  );
}
