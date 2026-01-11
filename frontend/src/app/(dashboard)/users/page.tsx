"use client";

import { useState } from "react";
import { useCan, useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PageLayout } from "@/components/PageLayout";
import {
  AdminUserWithRoles,
  NewAdminUser,
  NewAdminUserResponse,
  PermissionName,
  User,
} from "@/types";
import { UserPermissionsModal } from "./components/UserPermissionsModal";
import { UsersTable } from "./components/UsersTable";
import { Button } from "@mui/material";
import { UserModal } from "./components/UserModal";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { dataLayer } from "@/lib/dataLayer";
import { queryKeys } from "@/utils/query";
import { toast } from "react-toastify";

export default function UsersPage() {
  const { data, isFetching } = useList<AdminUserWithRoles>({
    endpoint: ENDPOINTS.USERS,
  });
  const [isPermissionsModalOpen, setIsPermissionsModalOpen] = useState(false);
  const [isUserModalOpen, setIsUserModalOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<AdminUserWithRoles | null>(
    null
  );
  const [tfaSecret, setTfaSecret] = useState<string | null>(null);
  const [tfaQrCode, setTfaQrCode] = useState<string | null>(null);
  const { can: canCreate } = useCan(PermissionName.AdminUsersCreate);
  const queryClient = useQueryClient();

  const openPermissionsModal = (user: AdminUserWithRoles) => {
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

  const createMutation = useMutation<NewAdminUserResponse, Error, NewAdminUser>(
    {
      mutationFn: (params) =>
        dataLayer.create({
          url: ENDPOINTS.USERS,
          params,
        }),
      onSuccess: (response) => {
        queryClient.invalidateQueries({
          queryKey: queryKeys.list(ENDPOINTS.USERS),
        });
        setTfaSecret(response.two_fa_secret);
        setTfaQrCode(response.two_fa_qr_code);
      },
      onError: (error) => toast.error(error.message),
    }
  );

  return (
    <PageLayout title="Управление администраторами">
      {canCreate && (
        <Button variant="contained" sx={{ mb: 2 }} onClick={openUserModal}>
          Создать пользователя
        </Button>
      )}
      <UsersTable
        users={data?.data || []}
        onConfigure={openPermissionsModal}
        loading={isFetching}
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
