"use client";

import { useDataGrid } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PageLayout } from "@/components/PageLayout";
import { AuditLogTable } from "./components/AuditLogTable";

export default function AuditLogPage() {
  const {
    rows,
    rowCount,
    loading,
    paginationModel,
    onPaginationModelChange,
    filterModel,
    onFilterModelChange,
  } = useDataGrid(ENDPOINTS.AUDIT_LOGS);

  return (
    <PageLayout title="Журнал аудита">
      <AuditLogTable
        logs={rows}
        loading={loading}
        rowCount={rowCount}
        paginationModel={paginationModel}
        onPaginationModelChange={onPaginationModelChange}
        filterModel={filterModel}
        onFilterModelChange={onFilterModelChange}
      />
    </PageLayout>
  );
}
