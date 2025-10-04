'use client';

import { List } from "@/components/List";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { TransactionsTable } from './components/TransactionsTable';

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
    <List title="Транзакции">
      <TransactionsTable transactions={transactions?.data || []} />
    </List>
  );
}