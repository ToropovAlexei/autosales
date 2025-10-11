"use client";

import { usePermissions } from "./usePermissions";

export const useCan = (permission: string) => {
  const { permissions, isLoading } = usePermissions();

  if (isLoading) {
    return false; // Or a loading state, but false is safer for rendering
  }

  return permissions.includes(permission);
};
