"use client";

import { useMemo, useState } from "react";
import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  CardHeader,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  Stack,
  TextField,
  Typography,
} from "@mui/material";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "react-toastify";
import { ENDPOINTS } from "@/constants";
import { PageLayout } from "@/components/PageLayout";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import {
  CreateBalanceRequest,
  PermissionName,
  PricingSettings,
  StoreBalance,
  StoreBalanceRequestType,
} from "@/types";
import { useCan, useOne } from "@/hooks";

const TRC20_WALLET_REGEX = /^T[1-9A-HJ-NP-Za-km-z]{33}$/;

const formatNumber = (value?: number, digits = 2) => {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return "н/д";
  }

  return value.toFixed(digits);
};

export default function BalanceManagementPage() {
  const [requestType, setRequestType] = useState<StoreBalanceRequestType>("deposit");
  const [walletAddress, setWalletAddress] = useState("");
  const [amountUsdt, setAmountUsdt] = useState("");
  const queryClient = useQueryClient();

  const { can: canManageBalance } = useCan(PermissionName.StoreBalanceWithdraw);

  const {
    data: storeBalance,
    isPending: isStoreBalancePending,
    error: storeBalanceError,
    refetch: refetchStoreBalance,
  } = useOne<StoreBalance>({
    endpoint: ENDPOINTS.STORE_BALANCE,
  });

  const {
    data: pricingSettings,
    isPending: isPricingPending,
    error: pricingError,
    refetch: refetchPricing,
  } = useOne<PricingSettings>({
    endpoint: ENDPOINTS.PRICING_SETTINGS,
  });

  const amountAsNumber = Number(amountUsdt);
  const usdtRateRub = pricingSettings?.usdt_rate_rub ?? 0;
  const amountRub =
    Number.isFinite(amountAsNumber) && amountAsNumber > 0 && usdtRateRub > 0
      ? amountAsNumber * usdtRateRub
      : 0;
  const balanceUsdt = usdtRateRub > 0 ? (storeBalance?.balance ?? 0) / usdtRateRub : 0;

  const validationMessage = useMemo(() => {
    if (!canManageBalance) {
      return "Недостаточно прав для создания заявки";
    }

    if (!walletAddress.trim()) {
      return "Введите адрес кошелька USDT TRC20";
    }

    if (!TRC20_WALLET_REGEX.test(walletAddress.trim())) {
      return "Некорректный формат TRC20-адреса";
    }

    if (!Number.isFinite(amountAsNumber) || amountAsNumber <= 0) {
      return "Сумма в USDT должна быть больше 0";
    }

    if (!pricingSettings || pricingSettings.usdt_rate_rub <= 0) {
      return "Курс USDT недоступен";
    }

    if (requestType === "withdrawal" && amountRub > (storeBalance?.balance ?? 0)) {
      return "Недостаточно средств на балансе магазина";
    }

    return null;
  }, [
    amountAsNumber,
    amountRub,
    canManageBalance,
    pricingSettings,
    requestType,
    storeBalance?.balance,
    walletAddress,
  ]);

  const { mutate: createRequest, isPending: isCreateRequestPending } = useMutation({
    mutationFn: async (params: CreateBalanceRequest) =>
      dataLayer.create({
        url: ENDPOINTS.STORE_BALANCE,
        params,
      }),
    onSuccess: async () => {
      toast.success("Заявка создана");
      setAmountUsdt("");
      setWalletAddress("");
      await queryClient.invalidateQueries({
        queryKey: queryKeys.one(ENDPOINTS.STORE_BALANCE),
      });
      await queryClient.invalidateQueries({
        queryKey: queryKeys.one(ENDPOINTS.PRICING_SETTINGS),
      });
      refetchStoreBalance();
      refetchPricing();
    },
    onError: (error) => {
      if (error instanceof Error) {
        toast.error(error.message);
        return;
      }
      toast.error("Не удалось создать заявку");
    },
  });

  const onSubmit = () => {
    if (validationMessage) {
      toast.warning(validationMessage);
      return;
    }

    createRequest({
      request_type: requestType,
      wallet_address: walletAddress.trim(),
      amount_rub: Number(amountRub.toFixed(2)),
    });
  };

  const isPending = isStoreBalancePending || isPricingPending;

  return (
    <PageLayout title="Управление балансом">
      <Stack gap={2}>
        <Stack direction={{ xs: "column", md: "row" }} gap={2}>
          <Card sx={{ flex: 1 }}>
            <CardContent>
              <Typography variant="body2" color="text.secondary">
                Баланс магазина
              </Typography>
              <Typography variant="h6">
                {isStoreBalancePending
                  ? "Загрузка..."
                  : `${formatNumber(storeBalance?.balance)} RUB`}
              </Typography>
            </CardContent>
          </Card>
          <Card sx={{ flex: 1 }}>
            <CardContent>
              <Typography variant="body2" color="text.secondary">
                Баланс магазина в USDT
              </Typography>
              <Typography variant="h6">
                {isPending ? "Загрузка..." : `${formatNumber(balanceUsdt, 4)} USDT`}
              </Typography>
            </CardContent>
          </Card>
          <Card sx={{ flex: 1 }}>
            <CardContent>
              <Typography variant="body2" color="text.secondary">
                Курс конвертации в USDT
              </Typography>
              <Typography variant="h6">
                {isPricingPending
                  ? "Загрузка..."
                  : `${formatNumber(pricingSettings?.usdt_rate_rub, 4)} RUB`}
              </Typography>
            </CardContent>
          </Card>
        </Stack>

        {storeBalanceError || pricingError ? (
          <Alert severity="error">
            Не удалось получить баланс или курс. Обновите страницу позже.
          </Alert>
        ) : null}

        <Card>
          <CardHeader title="Создать заявку" />
          <CardContent>
            <Stack gap={2}>
              <FormControl fullWidth>
                <InputLabel id="request-type-label">Тип заявки</InputLabel>
                <Select
                  labelId="request-type-label"
                  value={requestType}
                  label="Тип заявки"
                  onChange={(e) =>
                    setRequestType(e.target.value as StoreBalanceRequestType)
                  }
                >
                  <MenuItem value="deposit">Пополнение</MenuItem>
                  <MenuItem value="withdrawal">Вывод</MenuItem>
                </Select>
              </FormControl>

              <TextField
                label="Адрес кошелька в USDT TRC20"
                value={walletAddress}
                placeholder="T..."
                onChange={(e) => setWalletAddress(e.target.value)}
                fullWidth
              />

              <TextField
                label="Сумма в USDT"
                type="number"
                value={amountUsdt}
                onChange={(e) => setAmountUsdt(e.target.value)}
                inputProps={{ min: 0, step: "0.0001" }}
                fullWidth
              />

              <Box>
                <Typography variant="body2" color="text.secondary">
                  Эквивалент в RUB: {formatNumber(amountRub)}
                </Typography>
              </Box>

              {validationMessage ? (
                <Alert severity="warning">{validationMessage}</Alert>
              ) : null}

              <Button
                variant="contained"
                onClick={onSubmit}
                disabled={!!validationMessage || isCreateRequestPending}
                sx={{ width: "fit-content" }}
              >
                Создать заявку
              </Button>
            </Stack>
          </CardContent>
        </Card>
      </Stack>
    </PageLayout>
  );
}
