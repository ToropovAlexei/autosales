"use client";

import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PageLayout } from "@/components/PageLayout";
import { Role } from "@/types";
import { Button } from "@mui/material";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { ConfirmModal } from "@/components";
import { useState } from "react";
import { RoleModal } from "./components/RoleModal";
import { RolesTable } from "./components/RolesTable";

export default function RolesPage() {
  const queryClient = useQueryClient();
  const { data: roles, isPending } = useList<Role>({
    endpoint: ENDPOINTS.ROLES,
  });
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [selectedRole, setSelectedRole] = useState<Role | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingRole, setEditingRole] = useState<Role | null>(null);

  const deleteMutation = useMutation({
    mutationFn: (id: number) =>
      dataLayer.delete({ url: `${ENDPOINTS.ROLES}/${id}` }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ROLES),
      });
    },
  });

  const createMutation = useMutation({
    mutationFn: (params: { name: string; permissions: number[] }) =>
      dataLayer.create<{ id: number }>({
        url: ENDPOINTS.ROLES,
        params: { name: params.name },
      }),
    onSuccess: async (data, variables) => {
      const promises = variables.permissions.map((permissionId) =>
        addPermissionMutation.mutateAsync({
          role_id: data.id,
          permissionId,
        })
      );

      await Promise.all(promises);
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ROLES),
      });
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ROLE_PERMISSIONS),
      });
      setIsModalOpen(false);
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, name }: { id: number; name: string }) =>
      dataLayer.update({ url: ENDPOINTS.ROLES, id, params: { name } }),
    onSuccess: (data, variables) => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ROLES),
      });
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ROLE_PERMISSIONS),
      });
      setIsModalOpen(false);
      setEditingRole(null);
    },
  });

  const addPermissionMutation = useMutation({
    mutationFn: (params: { role_id: number; permissionId: number }) =>
      dataLayer.create({
        url: `${ENDPOINTS.ROLES}/${params.role_id}/permissions`,
        params: { permission_id: params.permissionId },
      }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ROLE_PERMISSIONS),
      });
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ROLES),
      });
    },
  });

  const removePermissionMutation = useMutation({
    mutationFn: ({
      role_id,
      permissionId,
    }: {
      role_id: number;
      permissionId: number;
    }) =>
      dataLayer.delete({
        url: `${ENDPOINTS.ROLES}/${role_id}/permissions/${permissionId}`,
      }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ROLE_PERMISSIONS),
      });
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ROLES),
      });
    },
  });

  const openConfirmDialog = (role: Role) => {
    setSelectedRole(role);
    setIsConfirmOpen(true);
  };

  const closeConfirmDialog = () => {
    setSelectedRole(null);
    setIsConfirmOpen(false);
  };

  const handleDelete = () => {
    if (selectedRole) {
      deleteMutation.mutate(selectedRole.id);
      closeConfirmDialog();
    }
  };

  const openModal = (role: Role | null) => {
    setEditingRole(role);
    setIsModalOpen(true);
  };

  const closeModal = () => {
    setEditingRole(null);
    setIsModalOpen(false);
  };

  const handleSave = (
    name: string,
    permissions: number[],
    initialPermissions: number[]
  ) => {
    if (editingRole) {
      if (editingRole.name !== name) {
        updateMutation.mutate({ id: editingRole.id, name });
      }

      const toAdd = permissions.filter((p) => !initialPermissions.includes(p));
      const toRemove = initialPermissions.filter(
        (p) => !permissions.includes(p)
      );

      toAdd.forEach((permissionId) => {
        addPermissionMutation.mutate({
          role_id: editingRole.id,
          permissionId,
        });
      });

      toRemove.forEach((permissionId) => {
        removePermissionMutation.mutate({
          role_id: editingRole.id,
          permissionId,
        });
      });

      closeModal();
    } else {
      createMutation.mutate({ name, permissions });
    }
  };

  return (
    <PageLayout title="Управление ролями">
      <Button
        variant="contained"
        sx={{ mb: 2 }}
        onClick={() => openModal(null)}
      >
        Создать роль
      </Button>
      <RolesTable
        roles={roles?.data || []}
        onEdit={openModal}
        onDelete={openConfirmDialog}
        loading={isPending}
      />
      <ConfirmModal
        open={isConfirmOpen}
        onClose={closeConfirmDialog}
        title="Вы уверены?"
        onConfirm={handleDelete}
        contentText={`Вы уверены, что хотите удалить роль "${selectedRole?.name}"?`}
        confirmBtnColor="error"
        closeBtnText="Отмена"
        confirmBtnText="Удалить"
        loading={deleteMutation.isPending}
      />
      {isModalOpen && (
        <RoleModal
          open={isModalOpen}
          onClose={closeModal}
          onSave={handleSave}
          role={editingRole}
        />
      )}
    </PageLayout>
  );
}
