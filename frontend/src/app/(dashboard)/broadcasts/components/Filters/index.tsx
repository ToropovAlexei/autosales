import { InputDateTime, InputNumber, InputSelect } from "@/components";
import { ENDPOINTS } from "@/constants";
import { useList } from "@/hooks";
import { Bot } from "@/types";
import { Typography } from "@mui/material";
import classes from "./styles.module.css";

export const Filters = () => {
  const { data: bots } = useList<Bot>({ endpoint: ENDPOINTS.BOTS });

  return (
    <div className={classes.container}>
      <Typography variant="h6">Фильтр пользователей</Typography>
      <div
        style={{
          display: "grid",
          gap: "1rem",
          gridTemplateColumns: "1fr 1fr",
          height: "fit-content",
        }}
      >
        <InputNumber name="balance_min" label="Минимальный баланс" />
        <InputNumber name="balance_max" label="Максимальный баланс" />
        <InputDateTime name="registered_after" label="Дата регистрации с" />
        <InputDateTime name="registered_before" label="Дата регистрации по" />
        <InputDateTime name="last_seen_after" label="Последнее посещение с" />
        <InputDateTime name="last_seen_before" label="Последнее посещение по" />
        <InputSelect
          name="last_seen_with_bot"
          label="Название бота"
          options={bots?.data.map((bot) => ({
            value: bot.id,
            label: bot.username,
          }))}
          withNone
        />
      </div>
    </div>
  );
};
