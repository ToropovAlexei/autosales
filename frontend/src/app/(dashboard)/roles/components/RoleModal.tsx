"use client";

import { Dialog, DialogTitle, DialogContent, DialogActions, Button, TextField, FormGroup, FormControlLabel, Checkbox } from "@mui/material";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { Permission, Role } from "@/types";
import { useEffect, useState } from "react";

interface RoleModalProps {
  open: boolean;
  onClose: () => void;
  onSave: (name: string, permissions: number[]) => void;
  role?: Role | null;
}

export const RoleModal = ({ open, onClose, onSave, role }: RoleModalProps) => {
  const [name, setName] = useState(role?.name || "");
  const [selectedPermissions, setSelectedPermissions] = useState<number[]>([]);

  const { data: allPermissions } = useList<Permission>({
    endpoint: ENDPOINTS.PERMISSIONS,
  });

  const { data: rolePermissions } = useList<Permission>({
    endpoint: `${ENDPOINTS.ROLES}/${role?.id}/permissions`,
    enabled: !!role,
  });

  useEffect(() => {
    if (open) {
      setName(role?.name || "");
      if (rolePermissions?.data) {
        setSelectedPermissions(rolePermissions.data.map((p) => p.id));
      } else {
        setSelectedPermissions([]);
      }
    }
  }, [open, role, rolePermissions]);

  const handleSave = () => {
    onSave(name, selectedPermissions);
  };

  const handlePermissionChange = (permissionId: number) => {
    setSelectedPermissions((prev) =>
      prev.includes(permissionId)
        ? prev.filter((id) => id !== permissionId)
        : [...prev, permissionId]
    );
  };

  const groupedPermissions = allPermissions?.data?.reduce((acc, permission) => {
    const group = permission.group || 'Other';
    if (!acc[group]) {
      acc[group] = [];
    }
    acc[group].push(permission);
    return acc;
  }, {} as Record<string, Permission[]>);

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
        {groupedPermissions && Object.entries(groupedPermissions).map(([group, permissions]) => (
          <div key={group}>
            <h4>{group}</h4>
            <FormGroup>
              {permissions.map((permission) => (
                <FormControlLabel
                  key={permission.id}
                  control={
                    <Checkbox
                      checked={selectedPermissions.includes(permission.id)}
                      onChange={() => handlePermissionChange(permission.id)}
                    />
                  }
                  label={permission.name}
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

