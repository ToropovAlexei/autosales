"use client";

import { useDataGrid } from "@/hooks";
import { APP_ROUTES, ENDPOINTS } from "@/constants";
import { ConfirmModal } from "@/components";
import { queryKeys } from "@/utils/query";
import { dataLayer } from "@/lib/dataLayer";
import { BotUsersTable } from "./components/BotUsersTable";
import { PageLayout } from "@/components/PageLayout";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Button, Stack } from "@mui/material";
import Link from "next/link";
import { AppRoute, Customer, UpdateCustomer } from "@/types";

export default function BotUsersPage() {
  const queryClient = useQueryClient();
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<Customer | null>(null);

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
  } = useDataGrid<Customer>(ENDPOINTS.CUSTOMERS);

  const { mutate, isPending } = useMutation({
    mutationFn: ({
      id,
      params,
    }: {
      id: Customer["id"];
      params: UpdateCustomer;
    }) =>
      dataLayer.update({
        url: ENDPOINTS.CUSTOMERS,
        id,
        params,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.list(ENDPOINTS.CUSTOMERS),
      });
      setIsConfirmOpen(false);
      setSelectedUser(null);
    },
  });

  const openConfirmDialog = (user: Customer) => {
    setSelectedUser(user);
    setIsConfirmOpen(true);
  };

  const handleToggleBlock = () => {
    if (selectedUser) {
      mutate({
        id: selectedUser.id,
        params: {
          is_blocked: !selectedUser.is_blocked,
        },
      });
    }
  };

  const confirmText = selectedUser?.is_blocked
    ? `Вы действительно хотите разблокировать пользователя ${selectedUser?.telegram_id}?`
    : `Вы действительно хотите заблокировать пользователя ${selectedUser?.telegram_id}?`;

  const confirmBtnText = selectedUser?.is_blocked
    ? "Разблокировать"
    : "Заблокировать";

  return (
    <PageLayout title="Покупатели">
      <Stack direction="row" mb={2} gap={2}>
        <Button onClick={() => refetch()}>Обновить</Button>
        <Button
          LinkComponent={Link}
          href={APP_ROUTES[AppRoute.Broadcasts]}
          variant="outlined"
        >
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
        loading={isPending}
        confirmBtnColor={selectedUser?.is_blocked ? "success" : "error"}
      />
    </PageLayout>
  );
}
