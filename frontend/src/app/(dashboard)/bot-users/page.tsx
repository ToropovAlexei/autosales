"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { ConfirmModal } from "@/components";
import { queryKeys } from "@/utils/query";
import { dataLayer } from "@/lib/dataLayer";
import { BotUsersTable } from "./components/BotUsersTable";
import { PageLayout } from "@/components/PageLayout";

interface BotUser {
  id: number;
  telegram_id: number;
  balance: number;
}

export default function BotUsersPage() {
  const queryClient = useQueryClient();
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<BotUser | null>(null);

  const { data: botUsers, isFetching } = useList<BotUser>({
    endpoint: ENDPOINTS.BOT_USERS,
  });

  const deleteMutation = useMutation({
    mutationFn: (userId: number) =>
      dataLayer.delete({ url: ENDPOINTS.BOT_USERS, id: userId }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.BOT_USERS),
      });
      setIsConfirmOpen(false);
      setSelectedUser(null);
    },
  });

  const openConfirmDialog = (user: BotUser) => {
    setSelectedUser(user);
    setIsConfirmOpen(true);
  };

  const handleDeleteUser = () => {
    if (selectedUser) {
      deleteMutation.mutate(selectedUser.id);
    }
  };

  return (
    <PageLayout title="Пользователи бота">
      <BotUsersTable
        users={botUsers?.data || []}
        onDelete={openConfirmDialog}
        loading={isFetching}
      />

      <ConfirmModal
        open={isConfirmOpen}
        onClose={() => setIsConfirmOpen(false)}
        onConfirm={handleDeleteUser}
        title="Вы уверены?"
        contentText={`Вы действительно хотите удалить пользователя ${selectedUser?.telegram_id}? Это действие необратимо.`}
        closeBtnText="Отмена"
        confirmBtnText="Удалить"
        loading={deleteMutation.isPending}
        confirmBtnColor="error"
      />
    </PageLayout>
  );
}
