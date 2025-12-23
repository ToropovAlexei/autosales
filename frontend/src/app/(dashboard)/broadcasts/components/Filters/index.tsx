import { InputDateTime, InputNumber, InputSelect } from "@/components";
import { ENDPOINTS } from "@/constants";
import { useList } from "@/hooks";
import { Bot } from "@/types";
import { Typography } from "@mui/material";

export const Filters = () => {
  const { data: bots } = useList<Bot>({
    endpoint: ENDPOINTS.ADMIN_REFERRAL_BOTS,
  });

  return (
    <div>
      <Typography variant="h6">Фильтр пользователей</Typography>
      <div
        style={{
          display: "grid",
          gap: "1rem",
          gridTemplateColumns: "1fr 1fr",
        }}
      >
        <InputNumber name="balance_min" label="Минимальный баланс" />
        <InputNumber name="balance_max" label="Максимальный баланс" />
        <InputDateTime name="registered_after" label="Дата регистрации с" />
        <InputDateTime name="registered_before" label="Дата регистрации по" />
        <InputDateTime name="last_seen_after" label="Последнее посещение с" />
        <InputDateTime name="last_seen_before" label="Последнее посещение по" />
        <InputSelect
          name="bot_name"
          label="Название бота"
          options={bots?.data.map((bot) => ({
            value: bot.username,
            label: bot.username,
          }))}
          withNone
        />
      </div>
    </div>
  );
};
