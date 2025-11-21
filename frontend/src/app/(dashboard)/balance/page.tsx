"use client";

import { useState } from "react";
import {
  Card,
  CardContent,
  CardHeader,
  Typography,
  TextField,
  Button,
  InputLabel,
  Box,
} from "@mui/material";

export default function BalanceManagementPage() {
  const [depositAmount, setDepositAmount] = useState("");
  const [withdrawalAmount, setWithdrawalAmount] = useState("");

  const testCryptoWallet = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"; // Placeholder value

  const handleDepositSubmit = () => {
    console.log("Deposit request submitted with amount:", depositAmount);
    // Future: Integrate with backend API
  };

  const handleWithdrawalSubmit = () => {
    console.log("Withdrawal request submitted with amount:", withdrawalAmount);
    // Future: Integrate with backend API
  };

  return (
    <Box sx={{ p: 3, spacing: 3 }}>
      <Typography
        variant="h4"
        component="h1"
        sx={{ fontWeight: "bold", mb: 3 }}
      >
        Управление Балансом
      </Typography>

      {/* Deposit Form */}
      <Card sx={{ mb: 3 }}>
        <CardHeader
          title={<Typography variant="h6">Пополнить Баланс</Typography>}
          subheader={
            <Typography variant="body2" color="text.secondary">
              Создайте заявку на пополнение баланса.
            </Typography>
          }
        />
        <CardContent>
          <Box sx={{ display: "grid", gap: "1.5rem" }}>
            <Box>
              <InputLabel htmlFor="deposit-amount">Сумма</InputLabel>
              <TextField
                id="deposit-amount"
                type="number"
                placeholder="Введите сумму"
                value={depositAmount}
                onChange={(e) => setDepositAmount(e.target.value)}
                fullWidth
              />
            </Box>
            <Box>
              <InputLabel>Тестовый крипто-кошелек</InputLabel>
              <TextField
                type="text"
                value={testCryptoWallet}
                InputProps={{
                  readOnly: true,
                }}
                fullWidth
              />
              <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                Это тестовый адрес, реальные пополнения пока не обрабатываются.
              </Typography>
            </Box>
            <Button onClick={handleDepositSubmit} variant="contained" fullWidth>
              Создать заявку на пополнение
            </Button>
          </Box>
        </CardContent>
      </Card>

      {/* Withdrawal Form */}
      <Card>
        <CardHeader
          title={<Typography variant="h6">Вывести Баланс</Typography>}
          subheader={
            <Typography variant="body2" color="text.secondary">
              Создайте заявку на вывод средств.
            </Typography>
          }
        />
        <CardContent>
          <Box sx={{ display: "grid", gap: "1.5rem" }}>
            <Box>
              <InputLabel htmlFor="withdrawal-amount">Сумма</InputLabel>
              <TextField
                id="withdrawal-amount"
                type="number"
                placeholder="Введите сумму"
                value={withdrawalAmount}
                onChange={(e) => setWithdrawalAmount(e.target.value)}
                fullWidth
              />
            </Box>
            <Button
              onClick={handleWithdrawalSubmit}
              variant="contained"
              fullWidth
            >
              Создать заявку на вывод
            </Button>
          </Box>
        </CardContent>
      </Card>
    </Box>
  );
}
