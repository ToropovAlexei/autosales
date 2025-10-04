'use client';

import { DataGrid, GridColDef } from '@mui/x-data-grid';
import { ruRU } from '@mui/x-data-grid/locales';

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
}

export const StockMovementsTable = ({ movements }: StockMovementsTableProps) => {
  const columns: GridColDef[] = [
    { field: 'id', headerName: 'ID', width: 90 },
    { field: 'product_id', headerName: 'ID Товара', width: 120 },
    { field: 'type', headerName: 'Тип', width: 120 },
    { field: 'quantity', headerName: 'Количество', width: 120 },
    {
      field: 'created_at',
      headerName: 'Дата',
      width: 200,
      renderCell: (params) => new Date(params.value).toLocaleString(),
    },
    { field: 'description', headerName: 'Описание', flex: 1, valueGetter: (value) => value ?? 'N/A' },
  ];

  return (
    <div style={{ width: '100%' }}>
      <DataGrid
        rows={movements}
        columns={columns}
        density="compact"
        initialState={{
          pagination: {
            paginationModel: {
              pageSize: 25,
            },
          },
        }}
        localeText={ruRU.components.MuiDataGrid.defaultProps.localeText}
      />
    </div>
  );
};
