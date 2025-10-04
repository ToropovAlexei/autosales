'use client';

import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { TransactionsTable } from './components/TransactionsTable';
import { PageLayout } from '@/components/PageLayout';

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
  const { data: transactions, isPending } = useList<Transaction>({
    endpoint: ENDPOINTS.TRANSACTIONS,
  });

  if (isPending) return <div>Загрузка...</div>;

  return (
    <PageLayout title="Транзакции">
      <TransactionsTable transactions={transactions?.data || []} />
    </PageLayout>
  );
}