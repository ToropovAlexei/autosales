"use client";

import { PageLayout } from "@/components/PageLayout";
import { BotSettingsForm } from "./components/BotSettingsForm";
import { Stack } from "@mui/material";

export default function WelcomeMessagesPage() {
  return (
    <PageLayout title="Приветственные сообщения">
      <Stack gap={2}>
        <BotSettingsForm />
      </Stack>
    </PageLayout>
  );
}
