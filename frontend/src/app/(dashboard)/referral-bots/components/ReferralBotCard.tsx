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
  TextField,
  IconButton,
  Link,
} from "@mui/material";
import { Edit } from "@mui/icons-material";
import { useState } from "react";
import classes from "./styles.module.css";

interface ReferralBot {
  id: number;
  owner_telegram_id: number;
  token: string;
  username: string;
  created_at: string;
  is_active: boolean;
  is_primary: boolean;
  turnover: number;
  accruals: number;
  referral_percentage: number;
}

interface ReferralBotCardProps {
  bot: ReferralBot;
  onUpdateStatus: (opts: { botId: number; isActive: boolean }) => void;
  onSetPrimary: (botId: number) => void;
  onUpdatePercentage: (opts: { botId: number; percentage: number }) => void;
  onDelete: (botId: number) => void;
  isUpdatingStatus: boolean;
  isSettingPrimary: boolean;
}

export const ReferralBotCard = ({
  bot,
  onUpdateStatus,
  onSetPrimary,
  onUpdatePercentage,
  onDelete,
  isUpdatingStatus,
  isSettingPrimary,
}: ReferralBotCardProps) => {
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [percentage, setPercentage] = useState(bot.referral_percentage.toString());
  const [validationError, setValidationError] = useState("");

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

  const handleUpdatePercentage = () => {
    const percentageValue = parseFloat(percentage);
    if (isNaN(percentageValue) || percentageValue < 0 || percentageValue > 100) {
      setValidationError("Процент должен быть от 0 до 100");
      return;
    }
    onUpdatePercentage({ botId: bot.id, percentage: percentageValue });
    setIsEditing(false);
    setValidationError("");
  };

  const handlePercentageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setPercentage(e.target.value);
    if (validationError) {
      setValidationError("");
    }
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
          <Typography variant="body2" color="text.secondary">
            <Link
              href={`https://t.me/${bot.username}`}
              target="_blank"
              rel="noopener noreferrer"
            >
              @{bot.username}
            </Link>
          </Typography>
          <div>
            <Typography variant="body2" color="text.secondary">
              Токен: ...{bot.token.slice(-8)}
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Оборот: {bot.turnover.toFixed(2)} ₽
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Начисления: {bot.accruals.toFixed(2)} ₽
            </Typography>
            {isEditing ? (
              <div className={classes.editContainer}>
                <TextField
                  label="Процент"
                  type="number"
                  value={percentage}
                  onChange={handlePercentageChange}
                  size="small"
                  error={!!validationError}
                  helperText={validationError}
                />
                <Button onClick={handleUpdatePercentage} size="small">
                  Сохранить
                </Button>
                <Button onClick={() => setIsEditing(false)} size="small">
                  Отмена
                </Button>
              </div>
            ) : (
              <Stack direction="row" alignItems="center" gap={1}>
                <Typography variant="body2" color="text.secondary">
                  Процент: {bot.referral_percentage}%
                </Typography>
                <IconButton onClick={() => setIsEditing(true)} size="small">
                  <Edit fontSize="small" />
                </IconButton>
              </Stack>
            )}
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
