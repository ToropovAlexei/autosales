"use client";

import { ConfirmModal } from "@/components";
import {
  Card,
  CardHeader,
  CardContent,
  CardActions,
  Typography,
  Chip,
  Switch,
  Button,
  Stack,
} from "@mui/material";
import { useState } from "react";
import classes from "./styles.module.css";

interface ReferralBot {
  id: number;
  owner_telegram_id: number;
  bot_token: string;
  created_at: string;
  is_active: boolean;
  is_primary: boolean;
  turnover: number;
  accruals: number;
}

interface ReferralBotCardProps {
  bot: ReferralBot;
  onUpdateStatus: (opts: { botId: number; isActive: boolean }) => void;
  onSetPrimary: (botId: number) => void;
  onDelete: (botId: number) => void;
  isUpdatingStatus: boolean;
  isSettingPrimary: boolean;
}

export const ReferralBotCard = ({
  bot,
  onUpdateStatus,
  onSetPrimary,
  onDelete,
  isUpdatingStatus,
  isSettingPrimary,
}: ReferralBotCardProps) => {
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);

  const openConfirmDialog = () => {
    setIsConfirmOpen(true);
  };

  const closeConfirmDialog = () => {
    setIsConfirmOpen(false);
  };

  const handleDelete = () => {
    onDelete(bot.id);
    closeConfirmDialog();
  };

  return (
    <>
      <Card className={classes.container}>
        <CardHeader
          title={
            <Stack
              direction="row"
              justifyContent="space-between"
              alignItems="center"
            >
              Бот ID: {bot.id}
              <Stack direction="row" gap={1}>
                {bot.is_primary && (
                  <Chip label="Основной" color="primary" size="small" />
                )}
                <Chip
                  label={bot.is_active ? "Активен" : "Неактивен"}
                  color={bot.is_active ? "success" : "error"}
                  size="small"
                />
              </Stack>
            </Stack>
          }
          subheader={`TG ID владельца: ${bot.owner_telegram_id}`}
        />
        <CardContent>
          <div>
            <Typography variant="body2" color="text.secondary">
              Токен: ...{bot.bot_token.slice(-8)}
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Оборот: {bot.turnover.toFixed(2)} ₽
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Начисления: {bot.accruals.toFixed(2)} ₽
            </Typography>
          </div>
          <Stack direction="row" gap={1} alignItems="center" mt={1}>
            <Typography variant="subtitle2">Статус</Typography>
            <Switch
              checked={bot.is_active}
              onChange={(e) =>
                onUpdateStatus({ botId: bot.id, isActive: e.target.checked })
              }
              disabled={isUpdatingStatus}
            />
          </Stack>
        </CardContent>
        <CardActions>
          <Button
            variant="outlined"
            onClick={() => onSetPrimary(bot.id)}
            disabled={bot.is_primary || isSettingPrimary}
          >
            Сделать основным
          </Button>
          <Button variant="contained" color="error" onClick={openConfirmDialog}>
            Удалить
          </Button>
        </CardActions>
      </Card>
      <ConfirmModal
        open={isConfirmOpen}
        onClose={closeConfirmDialog}
        title="Вы уверены?"
        onConfirm={handleDelete}
        contentText="Это действие невозможно отменить. Бот будет удален навсегда."
        confirmBtnColor="error"
        closeBtnText="Отмена"
        confirmBtnText="Удалить"
      />
    </>
  );
};
