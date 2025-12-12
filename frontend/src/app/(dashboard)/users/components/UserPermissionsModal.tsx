import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  CircularProgress,
} from "@mui/material";
import { User, Role, UserPermission, Permission } from "@/types";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { useEffect, useState, useMemo } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { PermissionsSelector } from "@/components";
import { getUserPermissions } from "./utils";

interface UserPermissionsModalProps {
  open: boolean;
  onClose: () => void;
  user: User | null;
}

export const UserPermissionsModal = ({
  open,
  onClose,
  user,
}: UserPermissionsModalProps) => {
  const queryClient = useQueryClient();
  const [selectedRole, setSelectedRole] = useState<number | string>("");
  const [permissionOverrides, setPermissionOverrides] = useState<
    Record<number, "allow" | "deny">
  >({});

  // Fetch all possible roles and permissions
  const { data: allRoles, isPending: isRolesLoading } = useList<Role>({
    endpoint: ENDPOINTS.ROLES,
  });
  const { data: allPermissions, isPending: isPermissionsLoading } =
    useList<Permission>({ endpoint: ENDPOINTS.PERMISSIONS });

  // Fetch the specific permissions currently assigned to the user
  const { data: currentUserPermissions, isPending: isUserPermissionsLoading } =
    useList<UserPermission>({
      endpoint: ENDPOINTS.USER_PERMISSIONS,
      meta: { ":id": user?.id },
      enabled: !!user,
    });

  // Effect to initialize the state when the user prop changes
  useEffect(() => {
    if (user && currentUserPermissions?.data && allRoles?.data) {
      setSelectedRole(user.roles?.[0]?.id || "");
      const overrides: Record<number, "allow" | "deny"> = {};
      currentUserPermissions.data.forEach((p) => {
        overrides[p.permission_id] = p.effect;
      });
      setPermissionOverrides(overrides);
    }
  }, [user, currentUserPermissions, allRoles]);

  const setRoleMutation = useMutation({
    mutationFn: (roleId: number) =>
      dataLayer.update({
        url: ENDPOINTS.USER_ROLES,
        params: { role_id: roleId },
        meta: { ":id": user?.id },
      }),
  });

  const setUserPermissionMutation = useMutation({
    mutationFn: (params: { permissionId: number; effect: string }) =>
      dataLayer.create({
        url: `${ENDPOINTS.USERS}/${user?.id}/permissions`,
        params: { permission_id: params.permissionId, effect: params.effect },
      }),
  });

  const removeUserPermissionMutation = useMutation({
    mutationFn: (permissionId: number) =>
      dataLayer.delete({
        url: `${ENDPOINTS.USERS}/${user?.id}/permissions/${permissionId}`,
      }),
  });

  // Memoize the permissions derived from the selected role
  const rolePermissions = useMemo(() => {
    if (!selectedRole || !allRoles?.data) {
      return new Set<number>();
    }
    const role = allRoles.data.find((r) => r.id === selectedRole);
    // The backend now sends permissions with roles
    return new Set(role?.permissions?.map((p) => p.id) || []);
  }, [selectedRole, allRoles]);

  const handleSave = async () => {
    // 1. Update the role if it has changed
    const initialRoleId = user?.roles?.[0]?.id || "";
    if (selectedRole && selectedRole !== initialRoleId) {
      await setRoleMutation.mutateAsync(Number(selectedRole));
    }

    // 2. Calculate permission changes
    const originalOverrides: Record<number, "allow" | "deny"> = {};
    currentUserPermissions?.data.forEach((p) => {
      originalOverrides[p.permission_id] = p.effect;
    });

    const promises: Promise<any>[] = [];

    // Check for new or modified permissions
    for (const permIdStr in permissionOverrides) {
      const permId = Number(permIdStr);
      const newEffect = permissionOverrides[permId];
      const originalEffect = originalOverrides[permId];

      if (newEffect !== originalEffect) {
        promises.push(
          setUserPermissionMutation.mutateAsync({
            permissionId: permId,
            effect: newEffect,
          })
        );
      }
    }

    // Check for removed permissions
    for (const permIdStr in originalOverrides) {
      const permId = Number(permIdStr);
      if (permissionOverrides[permId] === undefined) {
        promises.push(removeUserPermissionMutation.mutateAsync(permId));
      }
    }

    await Promise.all(promises);

    // Invalidate queries to refetch data on next open
    queryClient.invalidateQueries({
      queryKey: queryKeys.list(ENDPOINTS.USERS),
    });
    queryClient.invalidateQueries({
      queryKey: queryKeys.list(ENDPOINTS.USER_PERMISSIONS),
    });
    setPermissionOverrides({});
    onClose();
  };

  const handlePermissionToggle = (permissionId: number, checked: boolean) => {
    setPermissionOverrides((prev) => {
      const newOverrides = { ...prev };
      newOverrides[permissionId] = checked ? "allow" : "deny";
      return newOverrides;
    });
  };

  const handleChangeAllPermissions = (checked: boolean) => {
    const newOverrides: Record<number, "allow" | "deny"> = {};
    const state = checked ? "allow" : "deny";
    allPermissions?.data.forEach(({ id }) => {
      newOverrides[id] = state;
    });
    setPermissionOverrides(newOverrides);
  };

  const handleRoleSelectChange = (roleId: number | string) => {
    setSelectedRole(roleId);
    // Clear local overrides when role changes, so UI reflects the new role's permissions
    setPermissionOverrides({});
  };

  if (!user) return null;

  const isLoading =
    isPermissionsLoading || isRolesLoading || isUserPermissionsLoading;

  return (
    <Dialog open={open} onClose={onClose} maxWidth="md">
      <DialogTitle>Настроить права для {user.login}</DialogTitle>
      <DialogContent>
        {isLoading ? (
          <CircularProgress />
        ) : (
          <>
            <FormControl fullWidth sx={{ my: 2 }}>
              <InputLabel>Роль</InputLabel>
              <Select
                value={selectedRole}
                label="Роль"
                onChange={(e) =>
                  handleRoleSelectChange(e.target.value as number)
                }
              >
                {allRoles?.data?.map((role) => (
                  <MenuItem key={role.id} value={role.id}>
                    {role.name}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>

            <h4>Права доступа:</h4>
            <PermissionsSelector
              value={getUserPermissions(
                allPermissions?.data || [],
                rolePermissions,
                permissionOverrides
              )}
              onChange={handlePermissionToggle}
              onChangeAll={handleChangeAllPermissions}
            />
          </>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Отмена</Button>
        <Button
          onClick={handleSave}
          disabled={
            setRoleMutation.isPending ||
            setUserPermissionMutation.isPending ||
            removeUserPermissionMutation.isPending
          }
        >
          Сохранить
        </Button>
      </DialogActions>
    </Dialog>
  );
};
