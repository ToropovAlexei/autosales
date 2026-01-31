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
import {
  Role,
  AdminUserWithRoles,
  UpdateUserPermissions,
  UserPermission,
  PermissionResponse,
} from "@/types";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { useEffect, useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { PermissionsSelector } from "@/components";
import { getUserPermissions } from "./utils";

interface UserPermissionsModalProps {
  open: boolean;
  onClose: () => void;
  user: AdminUserWithRoles | null;
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

  const role = user?.roles?.[0];
  const { data: rolePermissions, isLoading: isRolePermissionsLoading } =
    useList<PermissionResponse>({
      endpoint: ENDPOINTS.ROLE_PERMISSIONS,
      meta: { ":id": role?.id },
      enabled: !!role,
    });
  const { data: allRoles, isPending: isRolesLoading } = useList<Role>({
    endpoint: ENDPOINTS.ROLES,
  });
  const { data: allPermissions, isPending: isPermissionsLoading } =
    useList<PermissionResponse>({ endpoint: ENDPOINTS.PERMISSIONS });

  // Fetch the specific permissions currently assigned to the user
  const { data: currentUserPermissions, isPending: isUserPermissionsLoading } =
    useList<UserPermission>({
      endpoint: ENDPOINTS.USER_PERMISSIONS,
      meta: { ":id": user?.id },
      enabled: !!user,
    });

  // Effect to initialize the state when the user prop changes
  useEffect(() => {
    if (user && currentUserPermissions?.data) {
      setSelectedRole(user.roles?.[0]?.id || "");
      const overrides: Record<number, "allow" | "deny"> = {};
      currentUserPermissions.data.forEach((p) => {
        overrides[p.id] = p.effect;
      });
      setPermissionOverrides(overrides);
    }
  }, [user, currentUserPermissions]);

  const setRoleMutation = useMutation({
    mutationFn: (roleId: number) =>
      dataLayer.update({
        url: ENDPOINTS.USER_ROLES,
        params: { role_id: roleId },
        meta: { ":id": user?.id },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.USERS),
      });
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.USER_PERMISSIONS),
      });
    },
  });

  const setUserPermissionMutation = useMutation({
    mutationFn: (params: UpdateUserPermissions) =>
      dataLayer.update({
        url: ENDPOINTS.USER_PERMISSIONS,
        params,
        meta: { ":id": user?.id },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.USERS),
      });
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.USER_PERMISSIONS),
      });
    },
  });

  const handleSave = async () => {
    const initialRoleId = user?.roles?.[0]?.id || "";
    if (selectedRole && selectedRole !== initialRoleId) {
      await setRoleMutation.mutateAsync(Number(selectedRole));
    }

    const originalOverrides: Record<number, "allow" | "deny"> = {};
    currentUserPermissions?.data.forEach((p) => {
      originalOverrides[p.id] = p.effect;
    });

    const update: UpdateUserPermissions = { removed: [], upserted: [] };

    for (const permIdStr in permissionOverrides) {
      const permId = Number(permIdStr);
      const newEffect = permissionOverrides[permId];
      const originalEffect = originalOverrides[permId];

      if (newEffect !== originalEffect) {
        update.upserted.push({ id: permId, effect: newEffect });
      }
    }

    for (const permIdStr in originalOverrides) {
      const permId = Number(permIdStr);
      if (permissionOverrides[permId] === undefined && permId) {
        update.removed.push(permId);
      }
    }

    if (update.upserted.length || update.removed.length) {
      setUserPermissionMutation.mutate(update);
    }

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
    isPermissionsLoading ||
    isRolePermissionsLoading ||
    isUserPermissionsLoading ||
    isRolesLoading;

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
                new Set(rolePermissions?.data.map((p) => p.id) || []),
                permissionOverrides,
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
            setRoleMutation.isPending || setUserPermissionMutation.isPending
          }
        >
          Сохранить
        </Button>
      </DialogActions>
    </Dialog>
  );
};
