import { Permission, UserPermission } from "@/types";

export const getUserPermissions = (
  allPermissions: Permission[],
  rolePermissions: Set<number>,
  userPermissions: Record<number, UserPermission["effect"]>
) => {
  const result: number[] = [];

  allPermissions.forEach((permission) => {
    const effect = userPermissions[permission.id];
    const isAllowedToRole = rolePermissions.has(permission.id);
    const isAllowedToUser = effect === "allow";
    const isDeniedToUser = effect === "deny";

    if (!isDeniedToUser && (isAllowedToUser || isAllowedToRole)) {
      result.push(permission.id);
    }
  });

  return result;
};
