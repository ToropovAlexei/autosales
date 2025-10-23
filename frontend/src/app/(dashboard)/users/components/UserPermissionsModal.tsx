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
  FormControlLabel,
  RadioGroup,
  Radio,
  CircularProgress,
} from "@mui/material";
import { User, Role, Permission, UserPermission } from "@/types";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { useEffect, useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { translatePermission } from "@/lib/permissions";

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
  const [userPermissions, setUserPermissions] = useState<
    Record<number, "allow" | "deny" | "inherit">
  >({});

  const { data: allRoles } = useList<Role>({ endpoint: ENDPOINTS.ROLES });
  const { data: allPermissions, isPending: isPermissionsLoading } =
    useList<Permission>({ endpoint: ENDPOINTS.PERMISSIONS });
  const {
    data: currentUserPermissions,
    isPending: isUserPermissionsLoading,
    refetch,
  } = useList<UserPermission>({
    endpoint: `${ENDPOINTS.USERS}/${user?.id}/permissions`,
    enabled: !!user,
  });

  useEffect(() => {
    if (user) {
      setSelectedRole(user.roles?.[0]?.id || "");
      const permissions: Record<number, "allow" | "deny" | "inherit"> = {};
      if (currentUserPermissions?.data) {
        currentUserPermissions.data.forEach((p) => {
          permissions[p.permission_id] = p.effect;
        });
      }
      setUserPermissions(permissions);
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
    },
  });

  const setUserPermissionMutation = useMutation({
    mutationFn: (params: { permissionId: number; effect: string }) =>
      dataLayer.create({
        url: `${ENDPOINTS.USERS}/${user?.id}/permissions`,
        params: { permission_id: params.permissionId, effect: params.effect },
      }),
    onSuccess: () => {
      refetch();
    },
  });

  const removeUserPermissionMutation = useMutation({
    mutationFn: (permissionId: number) =>
      dataLayer.delete({
        url: `${ENDPOINTS.USERS}/${user?.id}/permissions/${permissionId}`,
      }),
    onSuccess: () => {
      refetch();
    },
  });

  const handleSave = async () => {
    if (selectedRole && typeof selectedRole === "number") {
      await setRoleMutation.mutateAsync(selectedRole);
    }

    const promises = Object.entries(userPermissions).map(
      ([permissionId, effect]) => {
        const originalEffect = currentUserPermissions?.data?.find(
          (p) => p.permission_id === Number(permissionId)
        )?.effect;

        if (effect === "inherit" && originalEffect) {
          return removeUserPermissionMutation.mutateAsync(Number(permissionId));
        } else if (effect !== "inherit" && effect !== originalEffect) {
          return setUserPermissionMutation.mutateAsync({
            permissionId: Number(permissionId),
            effect,
          });
        }
        return Promise.resolve();
      }
    );

    await Promise.all(promises);

    onClose();
  };

  const handlePermissionChange = (
    permissionId: number,
    effect: "allow" | "deny" | "inherit"
  ) => {
    setUserPermissions((prev) => ({ ...prev, [permissionId]: effect }));
  };

  if (!user) return null;

  const isLoading = isPermissionsLoading || isUserPermissionsLoading;

  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="md">
      <DialogTitle>Настроить права для {user.email}</DialogTitle>
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
                onChange={(e) => setSelectedRole(e.target.value as number)}
              >
                {allRoles?.data?.map((role) => (
                  <MenuItem key={role.id} value={role.id}>
                    {role.name}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>

            <h4>Индивидуальные права:</h4>
            {allPermissions?.data?.map((permission) => (
              <div
                key={permission.id}
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                  margin: "8px 0",
                }}
              >
                <span>{translatePermission(permission.name)}</span>
                <RadioGroup
                  row
                  value={userPermissions[permission.id] || "inherit"}
                  onChange={(e) =>
                    handlePermissionChange(permission.id, e.target.value as any)
                  }
                >
                  <FormControlLabel
                    value="inherit"
                    control={<Radio />}
                    label="Наследовать"
                  />
                  <FormControlLabel
                    value="allow"
                    control={<Radio />}
                    label="Разрешить"
                  />
                  <FormControlLabel
                    value="deny"
                    control={<Radio />}
                    label="Запретить"
                  />
                </RadioGroup>
              </div>
            ))}
          </>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Отмена</Button>
        <Button onClick={handleSave}>Сохранить</Button>
      </DialogActions>
    </Dialog>
  );
};
