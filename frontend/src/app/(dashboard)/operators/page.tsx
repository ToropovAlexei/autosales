"use client";

import { useDataGrid } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PageLayout } from "@/components/PageLayout";
import { Button, Stack } from "@mui/material";
import { PaymentInvoice } from "@/types";
import { InvoicesTable } from "./components/InvoicesTable";
import { InputSupportOperators } from "./components/InputSupportOperators";

const ACTIVE_INVOICE_STATUSES = [
  "pending",
  "processing",
  "awaiting_receipt",
  "receipt_submitted",
  "disputed",
];

export default function OperatorsPage() {
  const {
    rows: invoices,
    rowCount,
    loading: isFetching,
    paginationModel,
    onPaginationModelChange,
    filterModel,
    onFilterModelChange,
    sortModel,
    onSortModelChange,
    refetch,
  } = useDataGrid<PaymentInvoice>(ENDPOINTS.PAYMENT_INVOICES, {
    defaultFilterModel: {
      items: [
        {
          id: 1,
          field: "status",
          operator: "isAnyOf",
          value: ACTIVE_INVOICE_STATUSES,
        },
      ],
    },
  });

  return (
    <PageLayout title="Для операторов">
      <Stack gap={2} mb={2}>
        <InputSupportOperators />
        <Stack direction="row">
          <Button onClick={() => refetch()}>Обновить</Button>
        </Stack>
      </Stack>
      <InvoicesTable
        invoices={invoices}
        loading={isFetching}
        rowCount={rowCount}
        paginationModel={paginationModel}
        onPaginationModelChange={onPaginationModelChange}
        filterModel={filterModel}
        onFilterModelChange={onFilterModelChange}
        sortModel={sortModel}
        onSortModelChange={onSortModelChange}
      />
    </PageLayout>
  );
}
