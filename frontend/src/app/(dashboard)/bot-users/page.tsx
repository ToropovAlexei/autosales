"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import api from "@/lib/api";
import { List } from "@/components/List";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";

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
    mutationFn: (userId: number) => api.delete(`/admin/bot-users/${userId}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["bot-users"] });
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
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>ID</TableHead>
              <TableHead>Telegram ID</TableHead>
              <TableHead>Баланс</TableHead>
              <TableHead className="text-right">Действия</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {botUsers?.data?.map((botUser) => (
              <TableRow key={botUser.id}>
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

      {/* Confirmation Dialog */}
      <Dialog open={isConfirmOpen} onOpenChange={setIsConfirmOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Вы уверены?</DialogTitle>
            <DialogDescription>
              Вы действительно хотите удалить пользователя{" "}
              {selectedUser?.telegram_id}? Это действие необратимо.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setIsConfirmOpen(false)}>
              Отмена
            </Button>
            <Button
              variant="destructive"
              onClick={handleDeleteUser}
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending ? "Удаление..." : "Удалить"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
