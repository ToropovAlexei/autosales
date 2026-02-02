import { Order } from "@/types";
import {
  DataGrid,
  GridColDef,
  GridFilterModel,
  GridPaginationModel,
  GridSortModel,
} from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import { ORDER_STATUS_TRANSLATIONS } from "./constants";

interface OrdersTableProps {
  orders: Order[];
  loading: boolean;
  rowCount: number;
  paginationModel: GridPaginationModel;
  onPaginationModelChange: (model: GridPaginationModel) => void;
  filterModel: GridFilterModel;
  onFilterModelChange: (model: GridFilterModel) => void;
  sortModel: GridSortModel;
  onSortModelChange: (model: GridSortModel) => void;
}

export const OrdersTable = ({
  orders,
  loading,
  rowCount,
  paginationModel,
  onPaginationModelChange,
  filterModel,
  onFilterModelChange,
  sortModel,
  onSortModelChange,
}: OrdersTableProps) => {
  const columns: GridColDef<Order>[] = [
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
      field: "product_name",
      headerName: "Товар",
      flex: 1,
      sortable: false,
      valueGetter: (value, row) => row.order_items[0]?.name_at_purchase,
    },
    {
      field: "quantity",
      headerName: "Количество",
      width: 120,
      sortable: false,
      type: "number",
      valueGetter: (value, row) => row.order_items[0]?.quantity,
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
      flex: 1,
      sortable: false,
      valueFormatter: (value) => ORDER_STATUS_TRANSLATIONS[value] || value,
    },
    {
      field: "created_at",
      headerName: "Дата",
      width: 200,
      renderCell: (params) => new Date(params.value).toLocaleString(),
      sortable: false,
    },
  ];

  return (
    <div style={{ width: "100%" }}>
      <DataGrid
        rows={orders}
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
