'use client';

import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { List } from "@/components/List";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

interface Order {
  id: number;
  user_id: number;
  product_id: number;
  quantity: number;
  amount: number;
  status: string;
  created_at: string;
  user_telegram_id: number;
  product_name: string;
}

export default function OrdersPage() {
  const { data: orders, isPending } = useList<Order>({
    endpoint: ENDPOINTS.ORDERS,
  });

  if (isPending) return <div>Loading...</div>;

  return (
    <List title="Заказы">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>ID</TableHead>
            <TableHead>Telegram ID</TableHead>
            <TableHead>Товар</TableHead>
            <TableHead>Количество</TableHead>
            <TableHead>Сумма</TableHead>
            <TableHead>Статус</TableHead>
            <TableHead>Дата</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {orders?.data?.map((order) => (
            <TableRow key={order.id}>
              <TableCell>{order.id}</TableCell>
              <TableCell>{order.user_telegram_id}</TableCell>
              <TableCell>{order.product_name}</TableCell>
              <TableCell>{order.quantity}</TableCell>
              <TableCell>{order.amount} ₽</TableCell>
              <TableCell>{order.status}</TableCell>
              <TableCell>{new Date(order.created_at).toLocaleString()}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </List>
  );
}
