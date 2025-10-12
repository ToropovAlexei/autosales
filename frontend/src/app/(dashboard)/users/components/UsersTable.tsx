"use client";

import { DataGrid, GridColDef, GridActionsCellItem } from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import ManageAccountsIcon from "@mui/icons-material/ManageAccounts";
import { User } from "@/types";
import { useCan } from "@/hooks";

interface UsersTableProps {
  users: User[];
  onConfigure: (user: User) => void;
  loading: boolean;
}

export const UsersTable = ({
  users,
  onConfigure,
  loading,
}: UsersTableProps) => {
  const canConfigure = useCan("rbac:manage");

  const columns: GridColDef[] = [
    { field: "id", headerName: "ID", width: 90 },
    { field: "email", headerName: "Email", flex: 1 },
    {
      field: "roles",
      headerName: "Роли",
      flex: 1,
      valueGetter: (value) => value?.map((role: any) => role.name).join(", "),
    },
    {
      field: "actions",
      type: "actions",
      headerName: "Действия",
      width: 100,
      cellClassName: "actions",
      getActions: ({ row }) => {
        if (canConfigure) {
          return [
            <GridActionsCellItem
              key="configure"
              icon={<ManageAccountsIcon />}
              label="Configure"
              onClick={() => onConfigure(row)}
            />,
          ];
        }
        return [];
      },
    },
  ];

  return (
    <div style={{ width: "100%" }}>
      <DataGrid
        rows={users}
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
      />
    </div>
  );
};
