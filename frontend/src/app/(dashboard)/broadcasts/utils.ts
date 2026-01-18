import { IFilter } from "@/types";
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

export const formToFilters = ({
  balance_max,
  balance_min,
  last_seen_with_bot,
  last_seen_after,
  last_seen_before,
  registered_after,
  registered_before,
}: Partial<BroadcastForm>) => {
  return [
    { field: "balance", op: "gt", value: balance_min },
    { field: "balance", op: "lt", value: balance_max },
    { field: "last_seen_with_bot", op: "eq", value: last_seen_with_bot },
    { field: "last_seen_at", op: "gt", value: last_seen_after },
    { field: "last_seen_at", op: "lt", value: last_seen_before },
    { field: "created_at", op: "gt", value: registered_after },
    { field: "created_at", op: "lt", value: registered_before },
  ].filter(
    (f) => f.value !== undefined && f.value !== null && f.value !== "",
  ) satisfies IFilter["filters"];
};
