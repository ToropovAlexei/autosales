"use client";

import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  TextField,
  FormGroup,
  FormControlLabel,
  Checkbox,
} from "@mui/material";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { Permission, Role } from "@/types";
import { useEffect, useState } from "react";
import {
  translatePermission,
  translatePermissionGroup,
} from "@/lib/permissions";
import { PERMISSIONS_COLORS } from "./constants";

interface RoleModalProps {
  open: boolean;
  onClose: () => void;
  onSave: (
    name: string,
    permissions: number[],
    initialPermissions: number[]
  ) => void;
  role?: Role | null;
}

export const RoleModal = ({ open, onClose, onSave, role }: RoleModalProps) => {
  const [name, setName] = useState(role?.name || "");
  const [selectedPermissions, setSelectedPermissions] = useState<number[]>([]);

  const { data: allPermissions } = useList<Permission>({
    endpoint: ENDPOINTS.PERMISSIONS,
  });

  const { data: rolePermissions } = useList<Permission>({
    endpoint: ENDPOINTS.ROLE_PERMISSIONS,
    meta: { ":id": role?.id },
    enabled: !!role,
  });

  useEffect(() => {
    if (open) {
      setName(role?.name || "");
      setSelectedPermissions(
        rolePermissions?.data ? rolePermissions.data.map((p) => p.id) : []
      );
    }
  }, [open, role, rolePermissions]);

  const handleSave = () => {
    const initialPermissions = rolePermissions?.data?.map((p) => p.id) || [];
    onSave(name, selectedPermissions, initialPermissions);
  };

  const handlePermissionChange = (permissionId: number) => {
    setSelectedPermissions((prev) =>
      prev.includes(permissionId)
        ? prev.filter((id) => id !== permissionId)
        : [...prev, permissionId]
    );
  };

  const groupedPermissions = Object.groupBy(
    allPermissions?.data || [],
    (permission) => permission.group || "Other"
  );

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogTitle>{role ? "Редактировать роль" : "Создать роль"}</DialogTitle>
      <DialogContent>
        <TextField
          autoFocus
          margin="dense"
          label="Название роли"
          type="text"
          fullWidth
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        {groupedPermissions &&
          Object.entries(groupedPermissions).map(([group, permissions]) => (
            <div key={group}>
              <h4>{translatePermissionGroup(group)}</h4>
              <FormGroup>
                {permissions?.map((permission) => (
                  <FormControlLabel
                    key={permission.id}
                    control={
                      <Checkbox
                        checked={selectedPermissions.includes(permission.id)}
                        onChange={() => handlePermissionChange(permission.id)}
                      />
                    }
                    label={translatePermission(permission.name)}
                    slotProps={{
                      typography: {
                        color: PERMISSIONS_COLORS[permission.name] || "success",
                      },
                    }}
                  />
                ))}
              </FormGroup>
            </div>
          ))}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Отмена</Button>
        <Button onClick={handleSave}>Сохранить</Button>
      </DialogActions>
    </Dialog>
  );
};
