"use client";

import { useList } from "@/hooks/useList";
import { ENDPOINTS } from "@/constants";

export const usePermissions = () => {
  const { data, isPending, error } = useList<string>({
    endpoint: ENDPOINTS.ME_PERMISSIONS,
  });

  return {
    permissions: data?.data || [],
    isLoading: isPending,
    error,
  };
};
