"use client";

import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { IAuditLog } from "@/types";
import { PageLayout } from "@/components/PageLayout";
import { AuditLogTable } from "./components/AuditLogTable";

export default function AuditLogPage() {
  const { data: auditLogs, isPending: isLoading } = useList<IAuditLog>({
    endpoint: ENDPOINTS.AUDIT_LOGS,
  });

  if (isLoading) return <div>Loading...</div>;

  return (
    <PageLayout title="Журнал аудита">
      <AuditLogTable logs={auditLogs?.data || []} />
    </PageLayout>
  );
}
