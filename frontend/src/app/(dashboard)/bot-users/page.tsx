"use client";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
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
import { useAuth } from "@/contexts/AuthContext";

interface BotUser {
  id: number;
  telegram_id: number;
  balance: number;
}

export default function BotUsersPage() {
  const { user, loading: authLoading } = useAuth();
  const router = useRouter();
  const queryClient = useQueryClient();
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<BotUser | null>(null);

  const { data: botUsers, isLoading } = useQuery<BotUser[]>({
    queryKey: ["bot-users"],
    queryFn: () => api.get("/admin/bot-users"),
    enabled: !!user && user.role === "admin",
  });

  const deleteMutation = useMutation({
    mutationFn: (userId: number) => api.delete(`/admin/bot-users/${userId}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["bot-users"] });
      setIsConfirmOpen(false);
      setSelectedUser(null);
    },
  });

  useEffect(() => {
    if (!authLoading && (!user || user.role !== "admin")) {
      router.push("/categories");
    }
  }, [user, authLoading, router]);

  const openConfirmDialog = (user: BotUser) => {
    setSelectedUser(user);
    setIsConfirmOpen(true);
  };

  const handleDeleteUser = () => {
    if (selectedUser) {
      deleteMutation.mutate(selectedUser.id);
    }
  };

  if (authLoading || isLoading) return <div>Loading...</div>;

  if (!user || user.role !== "admin") {
    return null;
  }

  return (
    <div>
      <h1 className="text-2xl font-bold mb-4">Пользователи бота</h1>
      <div className="border rounded-lg">
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
            {botUsers?.map((botUser) => (
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
      </div>

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
    </div>
  );
}
