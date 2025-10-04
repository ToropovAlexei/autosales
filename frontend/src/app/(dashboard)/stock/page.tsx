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

interface StockMovement {
  id: number;
  product_id: number;
  type: string;
  quantity: number;
  created_at: string;
  description: string | null;
}

export default function StockPage() {
  const { data: movements, isLoading } = useList<StockMovement>({
    endpoint: ENDPOINTS.STOCK_MOVEMENTS,
  });

  if (isLoading) return <div>Загрузка...</div>;

  return (
    <List title="Движения по складу">
      <Table size="small">
        <TableHead>
          <TableRow>
            <TableCell>ID</TableCell>
            <TableCell>ID Товара</TableCell>
            <TableCell>Тип</TableCell>
            <TableCell>Количество</TableCell>
            <TableCell>Дата</TableCell>
            <TableCell>Описание</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {movements?.data?.map((movement) => (
            <TableRow hover key={movement.id}>
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
