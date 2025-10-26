"use client";

import { useDataGrid } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { StockMovementsTable } from "./components/StockMovementsTable";
import { PageLayout } from "@/components/PageLayout";

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
  } = useDataGrid(ENDPOINTS.STOCK_MOVEMENTS);

  return (
    <PageLayout title="Движения по складу">
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
