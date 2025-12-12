"use client";

import { PermissionName } from "@/types";
import { usePermissions } from "./usePermissions";

export const useCan = (permission: PermissionName) => {
  const { permissions, isLoading } = usePermissions();

  if (isLoading) {
    return { can: false, isLoading };
  }

  return { can: permissions.includes(permission), isLoading };
};
