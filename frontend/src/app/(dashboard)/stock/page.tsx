"use client";

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { List } from "@/components/List";
import { useList } from "@/hooks";
import { ENDPOINTS } from "@/constants";

interface StockMovement {
  id: number;
  product_id: number;
  type: string;
  quantity: number;
  created_at: string;
  description: string | null;
}

export default function StockPage() {
  const {
    data: movements,
    isLoading,
    error,
  } = useList<StockMovement>({
    endpoint: ENDPOINTS.STOCK_MOVEMENTS,
  });

  if (isLoading) return <div>Загрузка...</div>;
  if (error) return <div>Не удалось загрузить движения по складу</div>;

  return (
    <List title="Движения по складу">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>ID</TableHead>
            <TableHead>ID Товара</TableHead>
            <TableHead>Тип</TableHead>
            <TableHead>Количество</TableHead>
            <TableHead>Дата</TableHead>
            <TableHead>Описание</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {movements?.data?.map((movement) => (
            <TableRow key={movement.id}>
              <TableCell>{movement.id}</TableCell>
              <TableCell>{movement.product_id}</TableCell>
              <TableCell>{movement.type}</TableCell>
              <TableCell>{movement.quantity}</TableCell>
              <TableCell>
                {new Date(movement.created_at).toLocaleString()}
              </TableCell>
              <TableCell>{movement.description ?? "N/A"}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </List>
  );
}
