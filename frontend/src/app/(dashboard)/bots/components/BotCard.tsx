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
  Box,
} from "@mui/material";
import { Edit, ContentCopy as ContentCopyIcon } from "@mui/icons-material";
import { useState } from "react";
import classes from "./styles.module.css";
import { toast } from "react-toastify";
import { Bot, UpdateBot } from "@/types";

interface BotCardProps {
  bot: Bot;
  onUpdate: (opts: { id: Bot["id"]; params: UpdateBot }) => void;
  onDelete: (botId: Bot["id"]) => void;
  isPending: boolean;
}

export const BotCard = ({
  bot,
  isPending,
  onUpdate,
  onDelete,
}: BotCardProps) => {
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [percentage, setPercentage] = useState(
    bot.referral_percentage.toString()
  );
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
    if (
      isNaN(percentageValue) ||
      percentageValue < 0 ||
      percentageValue > 100
    ) {
      setValidationError("Процент должен быть от 0 до 100");
      return;
    }
    onUpdate({ id: bot.id, params: { referral_percentage: percentageValue } });
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
          title={bot.username}
          subheader={
            bot.owner_id
              ? `TG ID владельца: ${bot.owner_id}`
              : "Основной бот магазина"
          }
          action={
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
          }
        />
        <CardContent>
          <Stack direction="row" alignItems="center" gap={1} sx={{ mb: 1 }}>
            <Box
              sx={{
                overflow: "hidden",
                wordBreak: "break-all",
              }}
            >
              <Link
                href={`https://t.me/${bot.username}`}
                target="_blank"
                rel="noopener noreferrer"
                variant="body2"
              >
                https://t.me/{bot.username}
              </Link>
            </Box>
            <IconButton
              onClick={() =>
                navigator.clipboard
                  .writeText(`https://t.me/${bot.username}`)
                  .then(() => {
                    toast.success("Ссылка скопирована в буфер обмена");
                  })
              }
              size="small"
            >
              <ContentCopyIcon fontSize="small" />
            </IconButton>
          </Stack>
          <div>
            <Stack direction="row" alignItems="center" gap={1}>
              <Typography variant="body2" color="text.secondary">
                Токен: ...{bot.token.slice(-8)}
              </Typography>
              <IconButton
                onClick={() =>
                  navigator.clipboard.writeText(bot.token).then(() => {
                    toast.success("Токен скопирован в буфер обмена");
                  })
                }
                size="small"
              >
                <ContentCopyIcon fontSize="small" />
              </IconButton>
            </Stack>
            {/* TODO: add turnover and accruals */}
            {/* {bot.type === "referral" && (
              <>
                <Typography variant="body2" color="text.secondary">
                  Оборот: {bot.turnover.toFixed(2)} ₽
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  Начисления: {bot.accruals.toFixed(2)} ₽
                </Typography>
              </>
            )} */}
            {bot.type === "referral" && (
              <>
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
              </>
            )}
          </div>
          <Stack direction="row" gap={1} alignItems="center" mt={1}>
            <Typography variant="subtitle2">Статус</Typography>
            <Switch
              checked={bot.is_active}
              onChange={(e) =>
                onUpdate({
                  id: bot.id,
                  params: { is_active: e.target.checked },
                })
              }
              disabled={isPending}
            />
          </Stack>
        </CardContent>
        <CardActions>
          <Button
            variant="outlined"
            onClick={() =>
              onUpdate({
                id: bot.id,
                params: { is_primary: true },
              })
            }
            disabled={bot.is_primary || isPending}
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
