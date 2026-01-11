"use client";

import {
  DataGrid,
  GridColDef,
  GridFilterModel,
  GridPaginationModel,
} from "@mui/x-data-grid";
import { ruRU } from "@mui/x-data-grid/locales";
import { AuditLog } from "@/types";
import * as jsondiffpatch from "jsondiffpatch";
import * as formatters from "jsondiffpatch/formatters/console";

interface AuditLogTableProps {
  logs: AuditLog[];
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
  const columns: GridColDef<AuditLog>[] = [
    { field: "id", headerName: "Id", width: 90 },
    {
      field: "admin_user_login",
      headerName: "Логин",
    },
    {
      field: "action",
      headerName: "Действие",
      width: 200,
      sortable: false,
    },
    {
      field: "target",
      headerName: "Объект",
      width: 200,
      valueGetter: (_value, row) => `${row.target_table} (${row.target_id})`,
      filterable: false,
      sortable: false,
    },
    {
      field: "changes",
      headerName: "Изменения",
      width: 200,
      flex: 1,
      valueGetter: (_value, row) =>
        formatters.format(jsondiffpatch.diff(row.old_values, row.new_values)),
      sortable: false,
    },
    { field: "status", headerName: "Статус", width: 110, sortable: false },
    {
      field: "created_at",
      headerName: "Дата",
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
