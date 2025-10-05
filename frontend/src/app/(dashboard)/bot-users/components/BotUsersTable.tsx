"use client";

import { DataGrid, GridColDef, GridActionsCellItem } from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import BlockIcon from "@mui/icons-material/Block";
import LockOpenIcon from "@mui/icons-material/LockOpen";
import { Chip } from "@mui/material";
import { BotUser } from "@/types/common";

interface BotUsersTableProps {
  users: BotUser[];
  onToggleBlock: (user: BotUser) => void;
  loading: boolean;
}

export const BotUsersTable = ({
  users,
  onToggleBlock,
  loading,
}: BotUsersTableProps) => {
  const columns: GridColDef[] = [
    { field: "id", headerName: "ID", width: 60 },
    { field: "telegram_id", headerName: "Telegram ID", width: 150 },
    {
      field: "is_blocked",
      headerName: "Статус",
      width: 120,
      renderCell: (params) =>
        params.value ? (
          <Chip label="Заблокирован" color="error" size="small" />
        ) : (
          <Chip label="Активен" color="success" size="small" />
        ),
    },
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
            key="toggle-block"
            icon={row.is_blocked ? <LockOpenIcon /> : <BlockIcon />}
            label={row.is_blocked ? "Unblock" : "Block"}
            onClick={() => onToggleBlock(row)}
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
