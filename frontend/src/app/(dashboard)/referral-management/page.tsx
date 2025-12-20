"use client";

import { useOne } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PageLayout } from "@/components/PageLayout";
import { ReferralProgramForm } from "./components/ReferralProgramForm";
import { Stack } from "@mui/material";

interface Settings {
  [key: string]: string;
}

export default function ReferralManagementPage() {
  const {
    data: settings,
    isPending: isSettingsPending,
    refetch,
  } = useOne<Settings>({
    endpoint: ENDPOINTS.ADMIN_SETTINGS,
  });

  return (
    <PageLayout title="Управление рефералами">
      <Stack gap={2}>
        <ReferralProgramForm
          settings={settings}
          isSettingsPending={isSettingsPending}
          refetchSettings={refetch}
        />
      </Stack>
    </PageLayout>
  );
}
