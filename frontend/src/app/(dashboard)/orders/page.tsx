"use client";

import { useDataGrid } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { OrdersTable } from "./components/OrdersTable";
import { PageLayout } from "@/components/PageLayout";

export default function OrdersPage() {
  const {
    rows: orders,
    rowCount,
    loading: isFetching,
    paginationModel,
    onPaginationModelChange,
    filterModel,
    onFilterModelChange,
    sortModel,
    onSortModelChange,
  } = useDataGrid(ENDPOINTS.ORDERS);

  return (
    <PageLayout title="Покупки">
      <OrdersTable
        orders={orders}
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
