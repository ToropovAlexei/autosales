"use client";

import { DataGrid, GridColDef, GridActionsCellItem } from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import { IProduct } from "@/types";
import EditIcon from "@mui/icons-material/Edit";
import DeleteIcon from "@mui/icons-material/Delete";

interface ProductsTableProps {
  products: IProduct[];
  onEdit: (product: IProduct) => void;
  onDelete: (id: number) => void;
  getCategoryName: (id: number) => string;
}

export const ProductsTable = ({
  products,
  onEdit,
  onDelete,
  getCategoryName,
}: ProductsTableProps) => {
  const columns: GridColDef[] = [
    { field: "id", headerName: "ID", width: 90 },
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
    },
    {
      field: "category_id",
      headerName: "Категория",
      width: 200,
      valueGetter: (value) => getCategoryName(value),
      flex: 1,
    },
    { field: "price", headerName: "Цена", type: "number", width: 110 },
    {
      field: "stock",
      headerName: "Остаток",
      type: "number",
      width: 110,
      valueGetter: (value, row) => (row.type === "subscription" ? "∞" : value),
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
          <GridActionsCellItem
            key="delete"
            icon={<DeleteIcon />}
            label="Delete"
            onClick={() => onDelete(row.id)}
            disabled={!!row.provider}
          />,
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
        getRowClassName={(params) =>
          params.indexRelativeToCurrentPage % 2 === 0 ? 'even' : 'odd'
        }
        initialState={{
          pagination: {
            paginationModel: {
              pageSize: 25,
            },
          },
        }}
        getRowId={(row) =>
          row.provider ? `${row.provider}-${row.external_id}` : row.id
        }
        localeText={ruRU.components.MuiDataGrid.defaultProps.localeText}
        slotProps={{
          filterPanel: {
            filterFormProps: {
              logicOperatorInputProps: {
                variant: 'outlined',
                size: 'small',
              },
              columnInputProps: {
                variant: 'outlined',
                size: 'small',
                sx: { mt: 'auto' },
              },
              operatorInputProps: {
                variant: 'outlined',
                size: 'small',
                sx: { mt: 'auto' },
              },
              valueInputProps: {
                InputComponentProps: {
                  variant: 'outlined',
                  size: 'small',
                },
              },
            },
          },
        }}
      />
    </div>
  );
};
