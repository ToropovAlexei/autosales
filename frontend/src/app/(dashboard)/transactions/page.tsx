'use client';

import { useDataGrid } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { TransactionsTable } from './components/TransactionsTable';
import { PageLayout } from '@/components/PageLayout';

export default function TransactionsPage() {
  const {
    rows: transactions,
    rowCount,
    loading: isFetching,
    paginationModel,
    onPaginationModelChange,
    filterModel,
    onFilterModelChange,
    sortModel,
    onSortModelChange,
  } = useDataGrid(ENDPOINTS.TRANSACTIONS);

  return (
    <PageLayout title="Транзакции">
      <TransactionsTable
        transactions={transactions}
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