import { useState } from "react";
import { useList } from "./useList";
import {
  GridPaginationModel,
  GridFilterModel,
  GridSortModel,
} from "@mui/x-data-grid";

export const useDataGrid = <T>(endpoint: string) => {
  const [paginationModel, setPaginationModel] = useState({
    page: 0,
    pageSize: 10,
  });
  const [filterModel, setFilterModel] = useState<GridFilterModel>({
    items: [],
  });
  const [sortModel, setSortModel] = useState<GridSortModel>([
    { field: "id", sort: "asc" },
  ]);

  const formattedFilters = filterModel.items
    .filter((item) => item.value !== undefined && item.value !== "")
    .map(({ field, operator: op, value }) => ({ field, op, value }));

  const { data, isFetching, error, refetch } = useList<T>({
    endpoint,
    filter: {
      page: paginationModel.page + 1, // MUI is 0-indexed, backend is 1-indexed
      pageSize: paginationModel.pageSize,
      filters: formattedFilters.length > 0 ? formattedFilters : undefined,
      orderBy: sortModel[0]?.field,
      order: sortModel[0]?.sort,
    },
    placeholderData: (prev) => prev,
  });

  const onPaginationModelChange = (model: GridPaginationModel) => {
    setPaginationModel(model);
  };

  const onFilterModelChange = (model: GridFilterModel) => {
    setFilterModel(model);
  };

  const onSortModelChange = (model: GridSortModel) => {
    setSortModel(model);
  };

  return {
    rows: data?.data || [],
    rowCount: data?.total || 0,
    loading: isFetching,
    error,
    paginationModel,
    onPaginationModelChange,
    filterModel,
    onFilterModelChange,
    sortModel,
    onSortModelChange,
    refetch,
  };
};
