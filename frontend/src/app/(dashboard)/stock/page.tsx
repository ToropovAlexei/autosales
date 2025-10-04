"use client";

import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { StockMovementsTable } from "./components/StockMovementsTable";
import { PageLayout } from "@/components/PageLayout";

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
    <PageLayout title="Движения по складу">
      <StockMovementsTable movements={movements?.data || []} />
    </PageLayout>
  );
}
