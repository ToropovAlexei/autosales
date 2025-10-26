import {
  DataGrid,
  GridColDef,
  GridFilterModel,
  GridPaginationModel,
  GridSortModel,
} from "@mui/x-data-grid";
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
      headerName: "User ID",
      flex: 1,
      sortable: false,
      type: "number",
    },
    {
      field: "user_telegram_id",
      headerName: "Telegram ID",
      flex: 1,
      sortable: false,
      filterable: false,
      type: "number",
    },
    { field: "product_name", headerName: "Товар", flex: 1, sortable: false },
    {
      field: "quantity",
      headerName: "Количество",
      width: 120,
      sortable: false,
      type: "number",
    },
    {
      field: "amount",
      headerName: "Сумма",
      width: 120,
      renderCell: (params) => `${params.value} ₽`,
      sortable: false,
      type: "number",
    },
    { field: "status", headerName: "Статус", flex: 1, sortable: false },
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
