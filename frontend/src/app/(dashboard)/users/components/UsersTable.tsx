"use client";

import { DataGrid, GridColDef, GridActionsCellItem } from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import ManageAccountsIcon from "@mui/icons-material/ManageAccounts";
import { AdminUserWithRoles, PermissionName } from "@/types";
import { useCan } from "@/hooks";

interface UsersTableProps {
  users: AdminUserWithRoles[];
  onConfigure: (user: AdminUserWithRoles) => void;
  loading: boolean;
}

export const UsersTable = ({
  users,
  onConfigure,
  loading,
}: UsersTableProps) => {
  const { can: canConfigure } = useCan(PermissionName.RbacManage);

  const columns: GridColDef<AdminUserWithRoles>[] = [
    { field: "id", headerName: "Id", width: 90, sortable: false },
    { field: "login", headerName: "Логин", flex: 1, sortable: false },
    {
      field: "roles",
      headerName: "Роли",
      flex: 1,
      valueGetter: (value) =>
        (value as AdminUserWithRoles["roles"])
          ?.map((role) => role.name)
          .join(", "),
      sortable: false,
    },
    {
      field: "actions",
      type: "actions",
      headerName: "Действия",
      width: 100,
      cellClassName: "actions",
      sortable: false,
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
        localeText={ruRU.components.MuiDataGrid.defaultProps.localeText}
      />
    </div>
  );
};
