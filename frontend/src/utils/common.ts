import { IFilter } from "@/types";

export const serializeFilter = (filter: IFilter) => {
  const params = new URLSearchParams();

  const { page, page_size, order_by, order_dir, filters, ...rest } = filter;

  if (page) {
    params.append("page", String(filter.page));
  }

  if (page_size) {
    params.append("page_size", String(filter.page_size));
  }

  if (order_by) {
    params.append("order_by", order_by);
  }

  if (order_dir) {
    params.append("order", order_dir);
  }

  if (filters) {
    const mappedFilters = filters.map((f) => ({
      ...f,
      op: f.op === "is" ? "=" : f.op,
    }));
    mappedFilters.forEach((f, idx) => {
      params.append(`filters[${idx}][field]`, f.field);
      params.append(`filters[${idx}][op]`, f.op);
      params.append(`filters[${idx}][value]`, String(f.value));
    });
  }

  Object.entries(rest).forEach(([key, value]) => {
    params.append(key, String(value));
  });

  return params.toString();
};
