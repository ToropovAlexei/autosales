'use client';

import { DataGrid, GridColDef } from '@mui/x-data-grid';
import { ruRU } from '@mui/x-data-grid/locales';

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
}

export const TransactionsTable = ({ transactions }: TransactionsTableProps) => {
  const columns: GridColDef[] = [
    { field: 'id', headerName: 'ID', width: 90 },
    { field: 'user_id', headerName: 'ID Пользователя', width: 150 },
    { field: 'order_id', headerName: 'ID Заказа', width: 120, valueGetter: (value) => value ?? 'N/A' },
    { field: 'type', headerName: 'Тип', width: 120 },
    { field: 'amount', headerName: 'Сумма', width: 120 },
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
        rows={transactions}
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
