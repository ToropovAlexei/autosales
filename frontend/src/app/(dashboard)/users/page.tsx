"use client";

import { useState } from "react";
import { useDataGrid, useCan } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PageLayout } from "@/components/PageLayout";
import { User } from "@/types";
import { UserPermissionsModal } from "./components/UserPermissionsModal";
import { UsersTable } from "./components/UsersTable";
import { Button } from "@mui/material";
import { UserModal } from "./components/UserModal";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { toast } from "react-toastify";

export default function UsersPage() {
  const {
    rows,
    rowCount,
    loading,
    paginationModel,
    onPaginationModelChange,
    filterModel,
    onFilterModelChange,
  } = useDataGrid(ENDPOINTS.USERS);
  const [isPermissionsModalOpen, setIsPermissionsModalOpen] = useState(false);
  const [isUserModalOpen, setIsUserModalOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);
  const [tfaSecret, setTfaSecret] = useState<string | null>(null);
  const [tfaQrCode, setTfaQrCode] = useState<string | null>(null);
  const canCreate = useCan("users:create");
  const queryClient = useQueryClient();

  const openPermissionsModal = (user: User) => {
    setSelectedUser(user);
    setIsPermissionsModalOpen(true);
  };

  const closePermissionsModal = () => {
    setSelectedUser(null);
    setIsPermissionsModalOpen(false);
  };

  const openUserModal = () => {
    setIsUserModalOpen(true);
  };

  const closeUserModal = () => {
    setIsUserModalOpen(false);
    setTfaSecret(null);
    setTfaQrCode(null);
  };

  const createMutation = useMutation({
    mutationFn: (params: any) =>
      dataLayer.create<{ user: User; two_fa_secret: string; qr_code: string }>({
        url: ENDPOINTS.USERS,
        params,
      }),
    onSuccess: (response) => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.USERS),
      });
      setTfaSecret(response.two_fa_secret);
      setTfaQrCode(response.qr_code);
    },
    onError: (error) => toast.error(error.message),
  });

  return (
    <PageLayout title="Управление администраторами">
      {canCreate && (
        <Button variant="contained" sx={{ mb: 2 }} onClick={openUserModal}>
          Создать пользователя
        </Button>
      )}
      <UsersTable
        users={rows}
        onConfigure={openPermissionsModal}
        loading={loading}
        rowCount={rowCount}
        paginationModel={paginationModel}
        onPaginationModelChange={onPaginationModelChange}
        filterModel={filterModel}
        onFilterModelChange={onFilterModelChange}
      />
      {isPermissionsModalOpen && (
        <UserPermissionsModal
          open={isPermissionsModalOpen}
          onClose={closePermissionsModal}
          user={selectedUser}
        />
      )}
      {isUserModalOpen && (
        <UserModal
          open={isUserModalOpen}
          onClose={closeUserModal}
          onSave={createMutation.mutate}
          tfaSecret={tfaSecret}
          tfaQrCode={tfaQrCode}
        />
      )}
    </PageLayout>
  );
}
