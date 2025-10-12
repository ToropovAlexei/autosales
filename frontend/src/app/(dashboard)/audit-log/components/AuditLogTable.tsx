"use client";

import { DataGrid, GridColDef } from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import { IAuditLog } from "@/types";

interface AuditLogTableProps {
  logs: IAuditLog[];
}

export const AuditLogTable = ({ logs }: AuditLogTableProps) => {
  const columns: GridColDef[] = [
    { field: "id", headerName: "ID", width: 90 },
    { field: "user_email", headerName: "User", width: 250, flex: 1 },
    { field: "action", headerName: "Action", width: 200, flex: 1 },
    {
      field: "target",
      headerName: "Target",
      width: 200,
      valueGetter: (value, row) => `${row.target_type} (${row.target_id})`,
      flex: 1,
    },
    { field: "status", headerName: "Status", width: 110 },
    {
      field: "created_at",
      headerName: "Date",
      width: 200,
      valueGetter: (value) => new Date(value).toLocaleString(),
    },
  ];

  return (
    <div style={{ width: "100%" }}>
      <DataGrid
        rows={logs}
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
