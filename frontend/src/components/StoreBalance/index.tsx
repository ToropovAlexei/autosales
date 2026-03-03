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
    <Typography variant="body1">
      Баланс магазина: {data?.balance.toFixed(2)} USDT
    </Typography>
  );
};
