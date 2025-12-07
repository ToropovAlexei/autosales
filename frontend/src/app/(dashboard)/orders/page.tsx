"use client";

import { useDataGrid } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { OrdersTable } from "./components/OrdersTable";
import { PageLayout } from "@/components/PageLayout";
import { Button, Stack } from "@mui/material";

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
    refetch,
  } = useDataGrid(ENDPOINTS.ORDERS);

  return (
    <PageLayout title="Покупки">
      <Stack direction="row" mb={2}>
        <Button onClick={() => refetch()}>Обновить</Button>
      </Stack>
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
