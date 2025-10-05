"use client";

import { DataGrid, GridColDef, GridActionsCellItem } from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import DeleteIcon from "@mui/icons-material/Delete";
import { BotUser } from "@/types/common";

interface BotUsersTableProps {
  users: BotUser[];
  onDelete: (user: BotUser) => void;
  loading: boolean;
}

export const BotUsersTable = ({
  users,
  onDelete,
  loading,
}: BotUsersTableProps) => {
  const columns: GridColDef[] = [
    { field: "id", headerName: "ID", width: 60 },
    { field: "telegram_id", headerName: "Telegram ID", width: 150 },
    {
      field: "balance",
      headerName: "Баланс",
      width: 150,
      renderCell: (params) => `${params.value} ₽`,
    },
    {
      field: "registered_with_bot",
      headerName: "Бот регистрации",
      flex: 1,
    },
    {
      field: "last_seen_with_bot",
      headerName: "Последний бот",
      flex: 1,
    },
    {
      field: "has_passed_captcha",
      headerName: "Прошел капчу",
      type: "boolean",
      width: 150,
    },
    {
      field: "created_at",
      headerName: "Дата регистрации",
      width: 200,
      type: "dateTime",
      valueGetter: (value) => new Date(value),
    },
    {
      field: "last_seen_at",
      headerName: "Последний раз был",
      width: 200,
      type: "dateTime",
      valueGetter: (value) => new Date(value),
    },
    {
      field: "actions",
      type: "actions",
      headerName: "Действия",
      width: 100,
      cellClassName: "actions",
      getActions: ({ row }) => {
        return [
          <GridActionsCellItem
            key="delete"
            icon={<DeleteIcon />}
            label="Delete"
            onClick={() => onDelete(row)}
          />,
        ];
      },
    },
  ];

  return (
    <div style={{ width: "100%" }}>
      <DataGrid
        rows={users}
        columns={columns}
        density="compact"
        loading={loading}
        getRowClassName={(params) =>
          params.indexRelativeToCurrentPage % 2 === 0 ? "even" : "odd"
        }
        initialState={{
          pagination: {
            paginationModel: {
              pageSize: 25,
            },
          },
        }}
        localeText={ruRU.components.MuiDataGrid.defaultProps.localeText}
        slotProps={{
          filterPanel: {
            filterFormProps: {
              logicOperatorInputProps: {
                variant: "outlined",
                size: "small",
              },
              columnInputProps: {
                variant: "outlined",
                size: "small",
                sx: { mt: "auto" },
              },
              operatorInputProps: {
                variant: "outlined",
                size: "small",
                sx: { mt: "auto" },
              },
              valueInputProps: {
                InputComponentProps: {
                  variant: "outlined",
                  size: "small",
                },
              },
            },
          },
        }}
      />
    </div>
  );
};
