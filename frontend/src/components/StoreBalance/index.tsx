"use client";

import { CircularProgress, Typography } from "@mui/material";
import { useCan, useOne } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PermissionName } from "@/types";

export const StoreBalance = () => {
  const { data, isLoading, error } = useOne<{ current_balance: number }>({
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
    <Typography variant="h6" sx={{ mr: 2 }}>
      Баланс магазина: {data?.current_balance.toFixed(2)} RUB
    </Typography>
  );
};
