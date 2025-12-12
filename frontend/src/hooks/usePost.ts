import { ENDPOINTS } from "@/constants";
import { dataLayer } from "@/lib/dataLayer";
import {
  DefaultError,
  useMutation,
  UseMutationOptions,
  UseMutationResult,
} from "@tanstack/react-query";
import { toast } from "react-toastify";

const SUCCESS_MESSAGES_MAP: Record<string, string> = {
  [ENDPOINTS.CATEGORIES]: "Категория успешно создана",
  [ENDPOINTS.PRODUCTS]: "Товар успешно создан",
  [ENDPOINTS.USERS]: "Пользователь успешно создан",
  [ENDPOINTS.IMAGES]: "Изображение успешно загружено",
  [ENDPOINTS.ORDERS]: "Заказ успешно создан",
  [ENDPOINTS.ROLES]: "Роль успешно создана",
};

export const usePost = <
  TData = unknown,
  TError = DefaultError,
  TVariables extends {
    endpoint: string;
    params?: unknown;
    meta?: Record<string, unknown>;
  } = {
    endpoint: string;
    params?: unknown;
    meta?: Record<string, unknown>;
  },
  TContext = unknown
>(
  options?: Omit<
    UseMutationOptions<TData, TError, TVariables, TContext>,
    "mutationFn"
  >
): UseMutationResult<TData, TError, TVariables, TContext> =>
  useMutation<TData, TError, TVariables, TContext>({
    mutationFn: ({ endpoint, params, meta }) =>
      dataLayer.create<TData>({ url: endpoint, params, meta }),
    ...options,
    onSuccess: (data, variables, onMutateResult, context) => {
      const msg = SUCCESS_MESSAGES_MAP[variables.endpoint];
      if (msg) {
        toast.success(msg);
      }
      options?.onSuccess?.(data, variables, onMutateResult, context);
    },
  });
