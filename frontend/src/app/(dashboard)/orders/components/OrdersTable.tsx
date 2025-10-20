"use client";

import { DataGrid, GridColDef } from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";

interface Order {
  id: number;
  user_id: number;
  product_id: number;
  quantity: number;
  amount: number;
  status: string;
  created_at: string;
  user_telegram_id: number;
  product_name: string;
}

interface OrdersTableProps {
  orders: Order[];
  loading: boolean;
}

export const OrdersTable = ({ orders, loading }: OrdersTableProps) => {
  const columns: GridColDef[] = [
    { field: "id", headerName: "ID", width: 90 },
    { field: "user_telegram_id", headerName: "Telegram ID", flex: 1 },
    { field: "product_name", headerName: "Товар", flex: 1 },
    { field: "quantity", headerName: "Количество", width: 120 },
    {
      field: "amount",
      headerName: "Сумма",
      width: 120,
      renderCell: (params) => `${params.value} ₽`,
    },
    { field: "status", headerName: "Статус", flex: 1 },
    {
      field: "created_at",
      headerName: "Дата",
      width: 200,
      renderCell: (params) => new Date(params.value).toLocaleString(),
    },
  ];

  return (
    <div style={{ width: "100%" }}>
      <DataGrid
        rows={orders}
        columns={columns}
        density="compact"
        loading={loading}
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
