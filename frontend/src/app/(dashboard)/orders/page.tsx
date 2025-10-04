'use client';

import { useList } from '@/hooks';
import { ENDPOINTS } from '@/constants';
import { List } from '@/components/List';
import { OrdersTable } from './components/OrdersTable';

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
      <OrdersTable orders={orders?.data || []} />
    </List>
  );
}