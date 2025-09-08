import { FalsyValues } from "@/types";

export const compact = <T>(arr: T[]) =>
  arr.filter(Boolean) as Exclude<T, FalsyValues>[];
