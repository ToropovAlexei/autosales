import { IFilter } from "@/types";

export const serializeFilter = (filter: IFilter) => {
  const params = new URLSearchParams();

  if (filter.page) {
    params.append("page", String(filter.page));
  }

  if (filter.pageSize) {
    params.append("pageSize", String(filter.pageSize));
  }

  if (filter.orderBy) {
    params.append("orderBy", filter.orderBy);
  }

  if (filter.order) {
    params.append("order", filter.order);
  }

  if (filter.filters) {
    const mappedFilters = filter.filters.map((f) => ({
      ...f,
      op: f.op === "is" ? "=" : f.op,
    }));
    params.append("filters", JSON.stringify(mappedFilters));
  }

  return params.toString();
};
