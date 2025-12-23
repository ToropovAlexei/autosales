"use client";

import { useDataGrid } from "@/hooks";
import { APP_ROUTES, ENDPOINTS } from "@/constants";
import { ConfirmModal } from "@/components";
import { queryKeys } from "@/utils/query";
import { dataLayer } from "@/lib/dataLayer";
import { BotUsersTable } from "./components/BotUsersTable";
import { PageLayout } from "@/components/PageLayout";
import { BotUser } from "@/types/common";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Button, Stack } from "@mui/material";
import Link from "next/link";
import { AppRoute } from "@/types";

export default function BotUsersPage() {
  const queryClient = useQueryClient();
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<BotUser | null>(null);

  const {
    rows: botUsers,
    rowCount,
    loading: isFetching,
    paginationModel,
    onPaginationModelChange,
    filterModel,
    onFilterModelChange,
    sortModel,
    onSortModelChange,
    refetch,
  } = useDataGrid<BotUser>(ENDPOINTS.BOT_USERS);

  const toggleBlockMutation = useMutation({
    mutationFn: (user: BotUser) =>
      dataLayer.update({
        url: ENDPOINTS.TOGGLE_BLOCK,
        meta: { ":id": user.telegram_id },
      }),
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

  const handleToggleBlock = () => {
    if (selectedUser) {
      toggleBlockMutation.mutate(selectedUser);
    }
  };

  const confirmText = selectedUser?.is_blocked
    ? `Вы действительно хотите разблокировать пользователя ${selectedUser?.telegram_id}?`
    : `Вы действительно хотите заблокировать пользователя ${selectedUser?.telegram_id}?`;

  const confirmBtnText = selectedUser?.is_blocked
    ? "Разблокировать"
    : "Заблокировать";

  return (
    <PageLayout title="Пользователи бота">
      <Stack direction="row" mb={2} gap={2}>
        <Button onClick={() => refetch()}>Обновить</Button>
        <Button LinkComponent={Link} href={APP_ROUTES[AppRoute.Broadcasts]} variant="outlined">
          Сделать рекламную рассылку
        </Button>
      </Stack>
      <BotUsersTable
        users={botUsers}
        onToggleBlock={openConfirmDialog}
        loading={isFetching}
        rowCount={rowCount}
        paginationModel={paginationModel}
        onPaginationModelChange={onPaginationModelChange}
        filterModel={filterModel}
        onFilterModelChange={onFilterModelChange}
        sortModel={sortModel}
        onSortModelChange={onSortModelChange}
      />

      <ConfirmModal
        open={isConfirmOpen}
        onClose={() => setIsConfirmOpen(false)}
        onConfirm={handleToggleBlock}
        title="Вы уверены?"
        contentText={confirmText}
        closeBtnText="Отмена"
        confirmBtnText={confirmBtnText}
        loading={toggleBlockMutation.isPending}
        confirmBtnColor={selectedUser?.is_blocked ? "success" : "error"}
      />
    </PageLayout>
  );
}
