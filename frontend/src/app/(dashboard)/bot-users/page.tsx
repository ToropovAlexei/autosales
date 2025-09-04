"use client";

import { useQuery } from "@tanstack/react-query";
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
import { useRouter } from "next/navigation";
import { useEffect } from "react";

interface BotUser {
  id: number;
  telegram_id: number;
  balance: number;
}

export default function BotUsersPage() {
  const { user, loading: authLoading } = useAuth();
  const router = useRouter();

  const { data: botUsers, isLoading } = useQuery<BotUser[]>({
    queryKey: ["bot-users"],
    queryFn: () => api.get("/admin/bot-users"),
    enabled: !!user && user.role === 'admin',
  });

  useEffect(() => {
    if (!authLoading && (!user || user.role !== 'admin')) {
      router.push('/categories');
    }
  }, [user, authLoading, router]);

  if (authLoading || isLoading) return <div>Loading...</div>;
  
  if (!user || user.role !== 'admin') {
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
            </TableRow>
          </TableHeader>
          <TableBody>
            {botUsers?.map((botUser) => (
              <TableRow key={botUser.id}>
                <TableCell>{botUser.id}</TableCell>
                <TableCell>{botUser.telegram_id}</TableCell>
                <TableCell>{botUser.balance} ₽</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}