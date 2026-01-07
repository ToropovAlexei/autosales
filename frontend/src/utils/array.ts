import { FalsyValues } from "@/types";

export const compact = <T>(arr: T[]) =>
  arr.filter(Boolean) as Exclude<T, FalsyValues>[];

export const range = (start: number, end?: number, step = 1) => {
  const resEnd = end === undefined ? start : end;
  const resStart = end === undefined ? 0 : start;

  const result = [];

  for (let i = resStart; i < resEnd; i += step) {
    result.push(i);
  }

  return result;
};

export const keyBy = <T>(arr: T[], key: keyof T) =>
  arr.reduce((acc, item) => {
    acc[item[key] as string] = item;
    return acc;
  }, {} as Record<string, T>);
