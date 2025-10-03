"use client";

import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { List } from "@/components/List";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
} from "@mui/material";

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
      <Table size="small">
        <TableHead>
          <TableRow>
            <TableCell>ID</TableCell>
            <TableCell>Telegram ID</TableCell>
            <TableCell>Товар</TableCell>
            <TableCell>Количество</TableCell>
            <TableCell>Сумма</TableCell>
            <TableCell>Статус</TableCell>
            <TableCell>Дата</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {orders?.data?.map((order) => (
            <TableRow hover key={order.id}>
              <TableCell>{order.id}</TableCell>
              <TableCell>{order.user_telegram_id}</TableCell>
              <TableCell>{order.product_name}</TableCell>
              <TableCell>{order.quantity}</TableCell>
              <TableCell>{order.amount} ₽</TableCell>
              <TableCell>{order.status}</TableCell>
              <TableCell>
                {new Date(order.created_at).toLocaleString()}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </List>
  );
}
