import { IFilter } from "@/types";

export const serializeFilter = (filter: IFilter) => {
  const params = new URLSearchParams();

  const { page, pageSize, orderBy, order, filters, ...rest } = filter;

  if (page) {
    params.append("page", String(filter.page));
  }

  if (pageSize) {
    params.append("pageSize", String(filter.pageSize));
  }

  if (orderBy) {
    params.append("orderBy", orderBy);
  }

  if (order) {
    params.append("order", order);
  }

  if (filters) {
    const mappedFilters = filters.map((f) => ({
      ...f,
      op: f.op === "is" ? "=" : f.op,
    }));
    params.append("filters", JSON.stringify(mappedFilters));
  }

  Object.entries(rest).forEach(([key, value]) => {
    params.append(key, String(value));
  });

  return params.toString();
};
