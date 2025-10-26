"use client";

import {
  DataGrid,
  GridColDef,
  GridFilterModel,
  GridPaginationModel,
  GridSortModel,
} from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";

interface Transaction {
  id: number;
  user_id: number;
  order_id: number | null;
  type: string;
  amount: number;
  created_at: string;
  description: string | null;
}

interface TransactionsTableProps {
  transactions: Transaction[];
  loading: boolean;
  rowCount: number;
  paginationModel: GridPaginationModel;
  onPaginationModelChange: (model: GridPaginationModel) => void;
  filterModel: GridFilterModel;
  onFilterModelChange: (model: GridFilterModel) => void;
  sortModel: GridSortModel;
  onSortModelChange: (model: GridSortModel) => void;
}

export const TransactionsTable = ({
  transactions,
  loading,
  rowCount,
  paginationModel,
  onPaginationModelChange,
  filterModel,
  onFilterModelChange,
  sortModel,
  onSortModelChange,
}: TransactionsTableProps) => {
  const columns: GridColDef[] = [
    {
      field: "id",
      headerName: "ID",
      width: 90,
      sortable: false,
      type: "number",
    },
    {
      field: "user_id",
      headerName: "ID Пользователя",
      width: 150,
      sortable: false,
      type: "number",
    },
    {
      field: "order_id",
      headerName: "ID Заказа",
      width: 120,
      valueGetter: (value) => value ?? "N/A",
      sortable: false,
      type: "number",
    },
    { field: "type", headerName: "Тип", width: 120, sortable: false },
    {
      field: "amount",
      headerName: "Сумма",
      width: 120,
      sortable: false,
      type: "number",
    },
    {
      field: "created_at",
      type: "dateTime",
      headerName: "Дата",
      width: 200,
      valueGetter: (value) => new Date(value),
      sortable: false,
    },
    {
      field: "description",
      headerName: "Описание",
      flex: 1,
      valueGetter: (value) => value ?? "N/A",
      sortable: false,
    },
  ];

  return (
    <div style={{ width: "100%" }}>
      <DataGrid
        rows={transactions}
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
