import {
  DataGrid,
  GridColDef,
  GridActionsCellItem,
  GridFilterModel,
  GridPaginationModel,
  GridSortModel,
} from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import BlockIcon from "@mui/icons-material/Block";
import LockOpenIcon from "@mui/icons-material/LockOpen";
import { Chip } from "@mui/material";
import { BotUser } from "@/types/common";

interface BotUsersTableProps {
  users: BotUser[];
  onToggleBlock: (user: BotUser) => void;
  loading: boolean;
  rowCount: number;
  paginationModel: GridPaginationModel;
  onPaginationModelChange: (model: GridPaginationModel) => void;
  filterModel: GridFilterModel;
  onFilterModelChange: (model: GridFilterModel) => void;
  sortModel: GridSortModel;
  onSortModelChange: (model: GridSortModel) => void;
}

export const BotUsersTable = ({
  users,
  onToggleBlock,
  loading,
  rowCount,
  paginationModel,
  onPaginationModelChange,
  filterModel,
  onFilterModelChange,
  sortModel,
  onSortModelChange,
}: BotUsersTableProps) => {
  const columns: GridColDef[] = [
    { field: "id", headerName: "ID", width: 60, sortable: false },
    {
      field: "telegram_id",
      headerName: "Telegram ID",
      width: 150,
      sortable: false,
    },
    {
      field: "is_blocked",
      headerName: "Статус",
      width: 120,
      sortable: false,
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
      type: "number",
      width: 150,
      renderCell: (params) => `${params.value} ₽`,
      sortable: false,
    },
    {
      field: "registered_with_bot",
      headerName: "Бот регистрации",
      flex: 1,
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
      type: "dateTime",
      valueGetter: (value) => new Date(value),
      sortable: false,
    },
    {
      field: "actions",
      type: "actions",
      headerName: "Действия",
      width: 100,
      cellClassName: "actions",
      sortable: false,
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
        rowCount={rowCount}
        paginationModel={paginationModel}
        onPaginationModelChange={onPaginationModelChange}
        filterModel={filterModel}
        onFilterModelChange={onFilterModelChange}
        sortingMode="server"
        sortModel={sortModel}
        onSortModelChange={onSortModelChange}
        paginationMode="server"
        filterMode="server"
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
