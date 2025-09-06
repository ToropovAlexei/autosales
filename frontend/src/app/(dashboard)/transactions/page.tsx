'use client';

import { useQuery } from '@tanstack/react-query';
import api from '@/lib/api';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface Transaction {
  id: number;
  user_id: number;
  order_id: number | null;
  type: string;
  amount: number;
  created_at: string;
  description: string | null;
}

export default function TransactionsPage() {
  const { data: transactions, isLoading, error } = useQuery<Transaction[]>({
    queryKey: ['transactions'],
    queryFn: () => api.getTransactions(),
  });

  if (isLoading) return <div>Загрузка...</div>;
  if (error) return <div>Не удалось загрузить транзакции</div>;

  return (
    <Card>
      <CardHeader>
        <CardTitle>Транзакции</CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>ID</TableHead>
              <TableHead>ID Пользователя</TableHead>
              <TableHead>ID Заказа</TableHead>
              <TableHead>Тип</TableHead>
              <TableHead>Сумма</TableHead>
              <TableHead>Дата</TableHead>
              <TableHead>Описание</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {transactions?.map((transaction) => (
              <TableRow key={transaction.id}>
                <TableCell>{transaction.id}</TableCell>
                <TableCell>{transaction.user_id}</TableCell>
                <TableCell>{transaction.order_id ?? 'N/A'}</TableCell>
                <TableCell>{transaction.type}</TableCell>
                <TableCell>{transaction.amount}</TableCell>
                <TableCell>{new Date(transaction.created_at).toLocaleString()}</TableCell>
                <TableCell>{transaction.description ?? 'N/A'}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}