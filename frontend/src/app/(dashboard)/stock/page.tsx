"use client";

import { useDataGrid } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { StockMovementsTable } from "./components/StockMovementsTable";
import { PageLayout } from "@/components/PageLayout";
import { Button, Stack } from "@mui/material";
import { StockMovement } from "@/types";

export default function StockPage() {
  const {
    rows: movements,
    rowCount,
    loading: isFetching,
    paginationModel,
    onPaginationModelChange,
    filterModel,
    onFilterModelChange,
    sortModel,
    onSortModelChange,
    refetch,
  } = useDataGrid<StockMovement>(ENDPOINTS.STOCK_MOVEMENTS);

  return (
    <PageLayout title="Движения по складу">
      <Stack direction="row" mb={2}>
        <Button onClick={() => refetch()}>Обновить</Button>
      </Stack>
      <StockMovementsTable
        movements={movements}
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
