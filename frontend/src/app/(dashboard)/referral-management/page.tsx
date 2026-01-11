"use client";

import { PageLayout } from "@/components/PageLayout";
import { ReferralProgramForm } from "./components/ReferralProgramForm";
import { Stack } from "@mui/material";

export default function ReferralManagementPage() {
  return (
    <PageLayout title="Управление рефералами">
      <Stack gap={2}>
        <ReferralProgramForm />
      </Stack>
    </PageLayout>
  );
}
