"use client";

import {
  DataGrid,
  GridColDef,
  GridFilterModel,
  GridPaginationModel,
} from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import { IAuditLog } from "@/types";

interface AuditLogTableProps {
  logs: IAuditLog[];
  loading: boolean;
  rowCount: number;
  paginationModel: GridPaginationModel;
  onPaginationModelChange: (model: GridPaginationModel) => void;
  filterModel: GridFilterModel;
  onFilterModelChange: (model: GridFilterModel) => void;
}

export const AuditLogTable = ({
  logs,
  loading,
  rowCount,
  paginationModel,
  onPaginationModelChange,
  filterModel,
  onFilterModelChange,
}: AuditLogTableProps) => {
  const columns: GridColDef[] = [
    { field: "id", headerName: "Id", width: 90, sortable: false },
    {
      field: "user_login",
      headerName: "Логин",
      flex: 1,
    },
    {
      field: "action",
      headerName: "Действие",
      width: 200,
      flex: 1,
      sortable: false,
    },
    {
      field: "target",
      headerName: "Объект",
      width: 200,
      valueGetter: (value, row) => `${row.target_type} (${row.target_id})`,
      flex: 1,
      filterable: false,
      sortable: false,
    },
    {
      field: "changes",
      headerName: "Изменения",
      width: 200,
      flex: 1,
      valueGetter: (value, row) => JSON.stringify(row.changes),
      sortable: false,
    },
    { field: "status", headerName: "Статус", width: 110, sortable: false },
    {
      field: "created_at",
      headerName: "Дата",
      width: 200,
      valueGetter: (value) => new Date(value).toLocaleString(),
      sortable: false,
    },
  ];

  return (
    <div style={{ width: "100%" }}>
      <DataGrid
        rows={logs}
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
