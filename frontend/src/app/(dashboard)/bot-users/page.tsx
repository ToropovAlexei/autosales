"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Button } from "@/components/ui/button";

import { List } from "@/components/List";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
} from "@mui/material";
import { ConfirmModal } from "@/components";
import { queryKeys } from "@/utils/query";
import { dataLayer } from "@/lib/dataLayer";

interface BotUser {
  id: number;
  telegram_id: number;
  balance: number;
}

export default function BotUsersPage() {
  const queryClient = useQueryClient();
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<BotUser | null>(null);

  const { data: botUsers } = useList<BotUser>({
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
    <>
      <List title="Пользователи бота">
        <Table size="small">
          <TableHead>
            <TableRow>
              <TableCell>ID</TableCell>
              <TableCell>Telegram ID</TableCell>
              <TableCell>Баланс</TableCell>
              <TableCell className="text-right">Действия</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {botUsers?.data?.map((botUser) => (
              <TableRow hover key={botUser.id}>
                <TableCell>{botUser.id}</TableCell>
                <TableCell>{botUser.telegram_id}</TableCell>
                <TableCell>{botUser.balance} ₽</TableCell>
                <TableCell className="text-right">
                  <Button
                    variant="destructive"
                    onClick={() => openConfirmDialog(botUser)}
                  >
                    Удалить
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </List>

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
    </>
  );
}
