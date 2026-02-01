"use client";

import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PageLayout } from "@/components/PageLayout";
import { NewRole, Role, UpdateRole, UpdateRolePermissions } from "@/types";
import { Button } from "@mui/material";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { ConfirmModal } from "@/components";
import { useState } from "react";
import { RoleModal } from "./components/RoleModal";
import { RolesTable } from "./components/RolesTable";
import { toast } from "react-toastify";

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

  const { mutate: updatePermissions } = useMutation({
    mutationFn: ({
      id,
      params,
    }: {
      id: number;
      params: UpdateRolePermissions;
    }) =>
      dataLayer.update({
        url: ENDPOINTS.ROLE_PERMISSIONS,
        params,
        meta: { ":id": id },
      }),
    onSuccess: () => {
      toast.success("Настройки роли сохранены");
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ROLE_PERMISSIONS),
      });
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ROLES),
      });
    },
    onError: () => toast.error("Произошла ошибка"),
  });

  const createMutation = useMutation<
    Role,
    Error,
    { role: NewRole; permissions: number[] }
  >({
    mutationFn: ({ role }) =>
      dataLayer.create({
        url: ENDPOINTS.ROLES,
        params: role,
      }),
    onSuccess: (data, variables) => {
      toast.success(`Роль ${data.name} создана`);
      updatePermissions({
        id: data.id,
        params: {
          added: variables.permissions,
          removed: [],
        },
      });

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
    mutationFn: ({ id, params }: { id: number; params: UpdateRole }) =>
      dataLayer.update({ url: ENDPOINTS.ROLES, id, params }),
    onSuccess: () => {
      toast.success("Настройки роли сохранены");
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ROLES),
      });
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.ROLE_PERMISSIONS),
      });
      setIsModalOpen(false);
      setEditingRole(null);
    },
    onError: () => toast.error("Произошла ошибка"),
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
    initialPermissions: number[],
  ) => {
    closeModal();

    if (!editingRole) {
      createMutation.mutate({ role: { name }, permissions });
      return;
    }

    if (editingRole.name !== name) {
      updateMutation.mutate({
        id: editingRole.id,
        params: { name, description: null },
      });
    }

    const toAdd = permissions.filter((p) => !initialPermissions.includes(p));
    const toRemove = initialPermissions.filter((p) => !permissions.includes(p));

    updatePermissions({
      id: editingRole.id,
      params: { added: toAdd, removed: toRemove },
    });
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
