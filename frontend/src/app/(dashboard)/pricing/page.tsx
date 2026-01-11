"use client";

import { useOne } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PageLayout } from "@/components/PageLayout";
import { PaymentDiscountsForm } from "../settings/components/PaymentDiscountsForm";
import { Stack } from "@mui/material";
import { GlobalMarkupForm } from "../settings/components/GlobalMarkupForm";
import { PaymentSystemMarkupForm } from "../settings/components/PaymentSystemMarkupForm";
import { PricingSettings } from "@/types/settings";

export default function PricingPage() {
  const {
    data: settings,
    isPending: isSettingsPending,
    refetch,
  } = useOne<PricingSettings>({
    endpoint: ENDPOINTS.PRICING_SETTINGS,
  });

  return (
    <PageLayout title="Ценообразование">
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
      </Stack>
    </PageLayout>
  );
}
