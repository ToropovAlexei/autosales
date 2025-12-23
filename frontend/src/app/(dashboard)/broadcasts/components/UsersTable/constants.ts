import { GridColDef } from "@mui/x-data-grid";
import dayjs from "dayjs";

export const COLUMNS: GridColDef[] = [
  { field: "id", headerName: "ID", width: 60, sortable: false },
  {
    field: "telegram_id",
    headerName: "Telegram ID",
    width: 150,
    sortable: false,
  },
  {
    field: "balance",
    headerName: "Баланс",
    type: "number",
    width: 150,
    renderCell: (params) => `${params.value} ₽`,
    sortable: false,
  },
  {
    field: "last_seen_with_bot",
    headerName: "Последний бот",
    flex: 1,
    sortable: false,
  },
  {
    field: "created_at",
    headerName: "Дата регистрации",
    width: 200,
    type: "dateTime",
    valueGetter: (value) => new Date(value),
    sortable: false,
  },
  {
    field: "last_seen_at",
    headerName: "Последний раз был",
    width: 200,
    sortable: false,
    renderCell: (params) => dayjs(params.value).fromNow(),
  },
];
