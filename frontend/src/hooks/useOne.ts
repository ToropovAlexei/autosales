import { dataLayer } from "@/lib/dataLayer";
import { IFilter } from "@/types";
import { queryKeys } from "@/utils/query";
import { useQuery, UseQueryOptions } from "@tanstack/react-query";

interface IUseOneOptions<
  TQueryFnData = unknown,
  TError = unknown,
  TData = TQueryFnData
> extends Omit<
    UseQueryOptions<TQueryFnData, TError, TData>,
    "queryKey" | "queryFn"
  > {
  endpoint: string;
  id?: number | string;
  params?: IFilter;
}

export const useOne = <
  TQueryFnData = unknown,
  TError = unknown,
  TData = TQueryFnData
>({
  endpoint,
  id,
  params,
  meta,
  ...options
}: IUseOneOptions<TQueryFnData, TError, TData>) =>
  useQuery<TQueryFnData, TError, TData>({
    queryKey: queryKeys.one(endpoint, { id, params, meta }),
    queryFn: () => dataLayer.getOne<TQueryFnData>(endpoint, id, params, meta),
    ...options,
  });
