import { IFilter } from "@/types";
import { newApi } from "./api";
import { serializeFilter } from "@/utils";
import { ENDPOINT_UPDATE_PUT_EXCEPTIONS } from "@/constants";

const fillUrlWithMeta = (url: string, meta?: Record<string, unknown>) => {
  for (const key in meta) {
    url = url.replace(key, String(meta[key]));
  }

  return url;
};

class DataLayer {
  public getOne = async <T>(
    url: string,
    id?: string | number,
    filter?: IFilter,
    meta?: Record<string, unknown>
  ) => {
    const fullUrl = [
      fillUrlWithMeta(id ? `${url}/${id}` : url, meta),
      new URLSearchParams(serializeFilter(filter || {})).toString(),
    ]
      .filter(Boolean)
      .join("?");
    const response = await newApi.get(fullUrl).json<{ data: T }>();
    return response.data;
  };

  public getList = async <T>(
    url: string,
    filter: IFilter | undefined,
    meta?: Record<string, unknown>
  ) => {
    const fullUrl = [
      fillUrlWithMeta(url, meta),
      new URLSearchParams(serializeFilter(filter || {})).toString(),
    ]
      .filter(Boolean)
      .join("?");
    const response = await newApi.get(fullUrl).json<{ data: T[] }>();
    // TODO Total
    return { data: response.data, total: 0 };
  };

  public create = async <T>({
    url,
    params,
    meta,
  }: {
    url: string;
    params?: unknown;
    meta?: Record<string, unknown>;
  }) => {
    const response = await newApi
      .post(fillUrlWithMeta(url, meta), { json: params })
      .json<{ data: T }>();
    return response.data;
  };

  public update = async <T>({
    url,
    id,
    params,
    meta,
  }: {
    url: string;
    id?: string | number;
    params?: unknown;
    meta?: Record<string, unknown>;
  }) => {
    const baseUrl = fillUrlWithMeta(id ? `${url}/${id}` : url, meta);
    const method = ENDPOINT_UPDATE_PUT_EXCEPTIONS.has(url) ? "put" : "patch";
    const response = await newApi[method]<T>(baseUrl, { json: params }).json<{
      data: T;
    }>();
    return response.data;
  };
}

export const dataLayer = new DataLayer();
