"use client";

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
  const { data: transactions, isLoading } = useList<Transaction>({
    endpoint: ENDPOINTS.TRANSACTIONS,
  });

  if (isLoading) return <div>Загрузка...</div>;

  return (
    <List title="Транзакции">
      <Table size="small">
        <TableHead>
          <TableRow>
            <TableCell>ID</TableCell>
            <TableCell>ID Пользователя</TableCell>
            <TableCell>ID Заказа</TableCell>
            <TableCell>Тип</TableCell>
            <TableCell>Сумма</TableCell>
            <TableCell>Дата</TableCell>
            <TableCell>Описание</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {transactions?.data?.map((transaction) => (
            <TableRow hover key={transaction.id}>
              <TableCell>{transaction.id}</TableCell>
              <TableCell>{transaction.user_id}</TableCell>
              <TableCell>{transaction.order_id ?? "N/A"}</TableCell>
              <TableCell>{transaction.type}</TableCell>
              <TableCell>{transaction.amount}</TableCell>
              <TableCell>
                {new Date(transaction.created_at).toLocaleString()}
              </TableCell>
              <TableCell>{transaction.description ?? "N/A"}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </List>
  );
}
