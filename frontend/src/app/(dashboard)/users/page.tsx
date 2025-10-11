"use client";

import { useState } from "react";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { PageLayout } from "@/components/PageLayout";
import { User } from "@/types";
import { UserPermissionsModal } from "./components/UserPermissionsModal";
import { UsersTable } from "./components/UsersTable";

export default function UsersPage() {
  const { data: users, isPending } = useList<User>({
    endpoint: ENDPOINTS.USERS,
  });
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);

  const openModal = (user: User) => {
    setSelectedUser(user);
    setIsModalOpen(true);
  };

  const closeModal = () => {
    setSelectedUser(null);
    setIsModalOpen(false);
  };

  return (
    <PageLayout title="Управление администраторами">
      <UsersTable
        users={users?.data || []}
        onConfigure={openModal}
        loading={isPending}
      />
      {isModalOpen && (
        <UserPermissionsModal
          open={isModalOpen}
          onClose={closeModal}
          user={selectedUser}
        />
      )}
    </PageLayout>
  );
}

