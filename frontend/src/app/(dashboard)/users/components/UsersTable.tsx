"use client";

import {
  DataGrid,
  GridColDef,
  GridActionsCellItem,
  GridPaginationModel,
  GridFilterModel,
} from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import ManageAccountsIcon from "@mui/icons-material/ManageAccounts";
import { PermissionName, User } from "@/types";
import { useCan } from "@/hooks";

interface UsersTableProps {
  users: User[];
  onConfigure: (user: User) => void;
  loading: boolean;
  rowCount: number;
  paginationModel: GridPaginationModel;
  onPaginationModelChange: (model: GridPaginationModel) => void;
  filterModel: GridFilterModel;
  onFilterModelChange: (model: GridFilterModel) => void;
}

export const UsersTable = ({
  users,
  onConfigure,
  loading,
  rowCount,
  paginationModel,
  onPaginationModelChange,
  filterModel,
  onFilterModelChange,
}: UsersTableProps) => {
  const { can: canConfigure } = useCan(PermissionName.RbacManage);

  const columns: GridColDef[] = [
    { field: "id", headerName: "ID", width: 90, sortable: false },
    { field: "email", headerName: "Email", flex: 1, sortable: false },
    {
      field: "roles",
      headerName: "Роли",
      flex: 1,
      valueGetter: (value) => value?.map((role: any) => role.name).join(", "),
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
        rowCount={rowCount}
        paginationModel={paginationModel}
        onPaginationModelChange={onPaginationModelChange}
        filterModel={filterModel}
        onFilterModelChange={onFilterModelChange}
        paginationMode="server"
        filterMode="server"
        localeText={ruRU.components.MuiDataGrid.defaultProps.localeText}
      />
    </div>
  );
};
