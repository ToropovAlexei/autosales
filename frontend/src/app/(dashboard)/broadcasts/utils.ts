import { BroadcastForm } from "./types";

export const cleanFilters = (filters: Partial<BroadcastForm>) => {
  const cleaned: Record<string, any> = {};
  for (const key in filters) {
    const value = filters[key as keyof typeof filters];
    if (value !== null && value !== "" && value !== undefined) {
      cleaned[key] = value;
    }
  }
  return cleaned;
};
