"use client";

import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { OrdersTable } from "./components/OrdersTable";
import { PageLayout } from "@/components/PageLayout";

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
  const { data: orders, isFetching } = useList<Order>({
    endpoint: ENDPOINTS.ORDERS,
  });

  return (
    <PageLayout title="Заказы">
      <OrdersTable orders={orders?.data || []} loading={isFetching} />
    </PageLayout>
  );
}
