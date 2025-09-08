import { IFilter } from "@/types";

export const serializeFilter = (filter: IFilter) => {
  const params = new URLSearchParams();

  for (const [key, value] of Object.entries(filter)) {
    if (value === undefined || value === null) {
      continue;
    }

    if (Array.isArray(value)) {
      value.forEach((item) => {
        if (item !== undefined && item !== null) {
          params.append(key, String(item));
        }
      });
      continue;
    }

    params.append(key, String(value));
  }

  return params.toString();
};
