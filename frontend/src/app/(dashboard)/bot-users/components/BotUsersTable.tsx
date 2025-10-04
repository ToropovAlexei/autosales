'use client';

import { DataGrid, GridColDef, GridActionsCellItem } from '@mui/x-data-grid';
import { ruRU } from '@mui/x-data-grid/locales';
import DeleteIcon from '@mui/icons-material/Delete';

interface BotUser {
  id: number;
  telegram_id: number;
  balance: number;
}

interface BotUsersTableProps {
  users: BotUser[];
  onDelete: (user: BotUser) => void;
}

export const BotUsersTable = ({ users, onDelete }: BotUsersTableProps) => {
  const columns: GridColDef[] = [
    { field: 'id', headerName: 'ID', width: 90 },
    { field: 'telegram_id', headerName: 'Telegram ID', flex: 1 },
    { field: 'balance', headerName: 'Баланс', width: 150, renderCell: (params) => `${params.value} ₽` },
    {
      field: 'actions',
      type: 'actions',
      headerName: 'Действия',
      width: 100,
      cellClassName: 'actions',
      getActions: ({ row }) => {
        return [
          <GridActionsCellItem
            key="delete"
            icon={<DeleteIcon />}
            label="Delete"
            onClick={() => onDelete(row)}
          />,
        ];
      },
    },
  ];

  return (
    <div style={{ width: '100%' }}>
      <DataGrid
        rows={users}
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
