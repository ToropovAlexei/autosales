import { dataLayer } from "@/lib/dataLayer";
import { IFilter } from "@/types";
import { queryKeys } from "@/utils/query";
import { useQuery, UseQueryOptions } from "@tanstack/react-query";

interface IUseListOptions<TQueryFnData = unknown>
  extends Omit<UseQueryOptions<TQueryFnData>, "queryKey" | "queryFn"> {
  endpoint: string;
  filter?: IFilter;
}

export const useList = <TQueryFnData = unknown>({
  endpoint,
  filter,
  meta,
  ...options
}: IUseListOptions<{ data: TQueryFnData[]; total: number }>) =>
  useQuery<{ data: TQueryFnData[]; total: number }>({
    queryKey: queryKeys.list(endpoint, { filter, meta }),
    queryFn: () => dataLayer.getList<TQueryFnData>(endpoint, filter, meta),
    ...options,
  });
