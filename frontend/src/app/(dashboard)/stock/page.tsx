'use client';

import { List } from "@/components/List";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { StockMovementsTable } from './components/StockMovementsTable';

interface StockMovement {
  id: number;
  product_id: number;
  type: string;
  quantity: number;
  created_at: string;
  description: string | null;
}

export default function StockPage() {
  const { data: movements, isPending } = useList<StockMovement>({
    endpoint: ENDPOINTS.STOCK_MOVEMENTS,
  });

  if (isPending) return <div>Загрузка...</div>;

  return (
    <List title="Движения по складу">
      <StockMovementsTable movements={movements?.data || []} />
    </List>
  );
}