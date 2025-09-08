import { IFilter } from "@/types";
import { compact } from "./array";

export const queryKeys = {
  one: (
    endpoint: string,
    params?: {
      id?: number | string;
      params?: IFilter;
      meta?: Record<string, unknown>;
    }
  ) => compact([endpoint, "one", params?.id, params?.params, params?.meta]),
  list: (
    endpoint: string,
    params?: { filter?: IFilter; meta?: Record<string, unknown> }
  ) => compact([endpoint, "list", params?.filter, params?.meta]),
};
