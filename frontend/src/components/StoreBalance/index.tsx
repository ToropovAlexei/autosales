"use client";

import { CircularProgress, Stack, Typography } from "@mui/material";
import { useCan, useOne } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PermissionName, StoreBalance as StoreBalanceResponse } from "@/types";

export const StoreBalance = () => {
  const { data, isLoading, error } = useOne<StoreBalanceResponse>({
    endpoint: ENDPOINTS.STORE_BALANCE,
  });

  const { can } = useCan(PermissionName.StoreBalanceRead);

  if (!can) {
    return null;
  }

  if (isLoading) {
    return <CircularProgress />;
  }

  if (error) {
    return <Typography>Ошибка получения баланса</Typography>;
  }

  return (
    <Typography variant="body1">
      Баланс магазина: {data?.balance.toFixed(6)} USDT
    </Typography>
  );
};
