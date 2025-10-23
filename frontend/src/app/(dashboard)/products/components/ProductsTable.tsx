"use client";

import {
  DataGrid,
  GridColDef,
  GridActionsCellItem,
  GridPaginationModel,
  GridFilterModel,
} from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import { IProduct } from "@/types";
import EditIcon from "@mui/icons-material/Edit";
import DeleteIcon from "@mui/icons-material/Delete";

interface ProductsTableProps {
  products: IProduct[];
  onEdit: (product: IProduct) => void;
  onDelete: (id: number) => void;
  getCategoryName: (id: number) => string;
  loading: boolean;
  rowCount: number;
  paginationModel: GridPaginationModel;
  onPaginationModelChange: (model: GridPaginationModel) => void;
  filterModel: GridFilterModel;
  onFilterModelChange: (model: GridFilterModel) => void;
  sortModel: GridSortModel;
  onSortModelChange: (model: GridSortModel) => void;
  categories: { id: number; name: string }[];
}

export const ProductsTable = ({
  products,
  onEdit,
  onDelete,
  getCategoryName,
  loading,
  rowCount,
  paginationModel,
  onPaginationModelChange,
  filterModel,
  onFilterModelChange,
  sortModel,
  onSortModelChange,
  categories,
}: ProductsTableProps) => {
  const columns: GridColDef[] = [
    { field: "id", headerName: "ID", width: 90, filterable: false }, // Not reliable for sorting/filtering
    { field: "name", headerName: "Название", width: 250, flex: 1 },
    {
      field: "type",
      headerName: "Тип",
      width: 200,
      valueGetter: (value, row) => {
        return row.provider
          ? `Внешний (${row.provider})`
          : row.type === "subscription"
          ? `Подписка (${row.subscription_period_days} дн.)`
          : "Товар";
      },
      flex: 1,
      type: "singleSelect",
      valueOptions: ["item", "subscription"],
    },
    {
      field: "category_id",
      headerName: "Категория",
      width: 200,
      valueFormatter: (value) => getCategoryName(value),
      flex: 1,
      type: "singleSelect",
      valueOptions: categories.map((c) => ({ value: c.id, label: c.name })),
    },
    { field: "price", headerName: "Цена", type: "number", width: 110 },
    {
      field: "stock",
      headerName: "Остаток",
      type: "number",
      width: 110,
      valueGetter: (value, row) => (row.type === "subscription" ? "∞" : value),
      filterable: false, // Stock is calculated and not a direct DB field
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
            key="edit"
            icon={<EditIcon />}
            label="Edit"
            onClick={() => onEdit(row)}
            disabled={!!row.provider}
          />,
          // TODO Soft delete
          // <GridActionsCellItem
          //   key="delete"
          //   icon={<DeleteIcon />}
          //   label="Delete"
          //   onClick={() => onDelete(row.id)}
          //   disabled={!!row.provider}
          // />,
        ];
      },
    },
  ];

  return (
    <div style={{ width: "100%" }}>
      <DataGrid
        rows={products}
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
        getRowId={(row) =>
          row.provider ? `${row.provider}-${row.external_id}` : row.id
        }
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
