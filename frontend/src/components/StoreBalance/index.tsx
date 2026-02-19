"use client";

import { CircularProgress, Stack, Typography } from "@mui/material";
import { useCan, useOne } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import {
  PermissionName,
  PricingSettings,
  StoreBalance as StoreBalanceResponse,
} from "@/types";

export const StoreBalance = () => {
  const { data, isLoading, error } = useOne<StoreBalanceResponse>({
    endpoint: ENDPOINTS.STORE_BALANCE,
  });

  const { can } = useCan(PermissionName.StoreBalanceRead);
  const { can: canReadPricing } = useCan(PermissionName.PricingRead);
  const { data: pricingSettings } = useOne<PricingSettings>({
    endpoint: ENDPOINTS.PRICING_SETTINGS,
    enabled: canReadPricing,
    retry: false,
  });

  if (!can) {
    return null;
  }

  if (isLoading) {
    return <CircularProgress />;
  }

  if (error) {
    return <Typography>Ошибка получения баланса</Typography>;
  }

  const rate = canReadPricing ? pricingSettings?.usdt_rate_rub : null;

  return (
    <Stack sx={{ mr: 2 }} alignItems="flex-end">
      <Typography variant="body1">
        Баланс магазина: {data?.balance.toFixed(2)} RUB
      </Typography>
      <Typography variant="caption" color="text.secondary">
        Курс конвертации в USDT:{" "}
        {typeof rate === "number" ? `${rate.toFixed(4)} RUB` : "н/д"}
      </Typography>
    </Stack>
  );
};
