import { DataGrid, GridColDef, GridActionsCellItem } from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import { Role } from "@/types";
import { useCan } from "@/hooks";

interface RolesTableProps {
  roles: Role[];
  onEdit: (role: Role) => void;
  onDelete: (role: Role) => void;
  loading: boolean;
}

export const RolesTable = ({ roles, onEdit, onDelete, loading }: RolesTableProps) => {
  const canEdit = useCan("rbac:manage");
  const canDelete = useCan("rbac:manage");

  const columns: GridColDef[] = [
    { field: "id", headerName: "ID", width: 90 },
    { field: "name", headerName: "Название", flex: 1 },
    {
      field: "actions",
      type: "actions",
      headerName: "Действия",
      width: 100,
      cellClassName: "actions",
      getActions: ({ row }) => {
        const actions = [];
        if (canEdit) {
          actions.push(
            <GridActionsCellItem
              key="edit"
              icon={<EditIcon />}
              label="Edit"
              onClick={() => onEdit(row)}
            />
          );
        }
        if (canDelete) {
          actions.push(
            <GridActionsCellItem
              key="delete"
              icon={<DeleteIcon />}
              label="Delete"
              color="error"
              onClick={() => onDelete(row)}
            />
          );
        }
        return actions;
      },
    },
  ];
  return (
    <div style={{ width: "100%" }}>
      <DataGrid
        rows={roles}
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
