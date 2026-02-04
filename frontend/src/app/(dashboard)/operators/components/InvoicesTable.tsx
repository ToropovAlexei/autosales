import { PaymentInvoice } from "@/types";
import {
  DataGrid,
  GridColDef,
  GridFilterModel,
  GridPaginationModel,
  GridSortModel,
} from "@mui/x-data-grid";
import { Box } from "@mui/material";
import { ruRU } from "@mui/x-data-grid/locales";
import { INVOICE_STATUS_TRANSLATIONS } from "./constants";

interface OrdersTableProps {
  invoices: PaymentInvoice[];
  loading: boolean;
  rowCount: number;
  paginationModel: GridPaginationModel;
  onPaginationModelChange: (model: GridPaginationModel) => void;
  filterModel: GridFilterModel;
  onFilterModelChange: (model: GridFilterModel) => void;
  sortModel: GridSortModel;
  onSortModelChange: (model: GridSortModel) => void;
}

export const InvoicesTable = ({
  invoices,
  loading,
  rowCount,
  paginationModel,
  onPaginationModelChange,
  filterModel,
  onFilterModelChange,
  sortModel,
  onSortModelChange,
}: OrdersTableProps) => {
  const columns: GridColDef<PaymentInvoice>[] = [
    {
      field: "id",
      headerName: "Id",
      width: 90,
      sortable: false,
      type: "number",
    },
    {
      field: "customer_id",
      headerName: "Покупатель",
      flex: 1,
      sortable: false,
      type: "number",
    },
    {
      field: "gateway_invoice_id",
      headerName: "Токен",
      flex: 2,
      sortable: false,
      renderCell: (params) => (
        <Box component="span" sx={{ fontFamily: "monospace" }}>
          {params.value}
        </Box>
      ),
    },
    {
      field: "gateway",
      headerName: "Платежная система",
      width: 140,
      sortable: false,
      valueFormatter: (value) => String(value || "").toUpperCase(),
    },
    {
      field: "amount",
      headerName: "Сумма",
      width: 120,
      renderCell: (params) => `${params.value} ₽`,
      sortable: false,
      type: "number",
    },
    {
      field: "status",
      headerName: "Статус",
      width: 180,
      sortable: false,
      valueFormatter: (value) => INVOICE_STATUS_TRANSLATIONS[value] || value,
    },
    {
      field: "created_at",
      headerName: "Дата",
      width: 200,
      renderCell: (params) => new Date(params.value).toLocaleString(),
      sortable: false,
    },
    {
      field: "expires_at",
      headerName: "Истекает",
      width: 200,
      renderCell: (params) => new Date(params.value).toLocaleString(),
      sortable: false,
    },
  ];

  return (
    <div style={{ width: "100%" }}>
      <DataGrid
        rows={invoices}
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
