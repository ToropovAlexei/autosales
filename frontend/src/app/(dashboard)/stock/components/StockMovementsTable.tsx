"use client";

import {
  DataGrid,
  GridColDef,
  GridFilterModel,
  GridPaginationModel,
  GridSortModel,
} from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";

interface StockMovement {
  id: number;
  product_id: number;
  type: string;
  quantity: number;
  created_at: string;
  description: string | null;
}

interface StockMovementsTableProps {
  movements: StockMovement[];
  loading: boolean;
  rowCount: number;
  paginationModel: GridPaginationModel;
  onPaginationModelChange: (model: GridPaginationModel) => void;
  filterModel: GridFilterModel;
  onFilterModelChange: (model: GridFilterModel) => void;
  sortModel: GridSortModel;
  onSortModelChange: (model: GridSortModel) => void;
}

export const StockMovementsTable = ({
  movements,
  loading,
  rowCount,
  paginationModel,
  onPaginationModelChange,
  filterModel,
  onFilterModelChange,
  sortModel,
  onSortModelChange,
}: StockMovementsTableProps) => {
  const columns: GridColDef[] = [
    { field: "id", headerName: "ID", width: 90, sortable: false },
    {
      field: "product_id",
      headerName: "ID Товара",
      width: 120,
      sortable: false,
    },
    { field: "type", headerName: "Тип", width: 120, sortable: false },
    {
      field: "quantity",
      headerName: "Количество",
      width: 120,
      sortable: false,
    },
    {
      field: "created_at",
      headerName: "Дата",
      width: 200,
      renderCell: (params) => new Date(params.value).toLocaleString(),
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
        rows={movements}
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
