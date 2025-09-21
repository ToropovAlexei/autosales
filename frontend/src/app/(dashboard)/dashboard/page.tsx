"use client";

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { useOne } from "@/hooks";
import { ENDPOINTS } from "@/constants";

interface DashboardStats {
  total_users: number;
  users_with_purchases: number;
  available_products: number;
}

interface SalesOverTime {
  products_sold: number;
  total_revenue: number;
}

const getInitialStartDate = () => {
  const lastWeek = new Date();
  lastWeek.setDate(lastWeek.getDate() - 7);
  return lastWeek.toISOString();
};

const getInitialEndDate = () => {
  return new Date().toISOString();
};

export default function DashboardPage() {
  const [startDate, setStartDate] = useState(getInitialStartDate());
  const [endDate, setEndDate] = useState(getInitialEndDate());

  const { data: stats, isPending: isStatsPending } = useOne<DashboardStats>({
    endpoint: ENDPOINTS.DASHBOARD_STATS,
  });

  const { data: sales, isPending: isSalesPending } = useOne<SalesOverTime>({
    endpoint: ENDPOINTS.SALES_OVER_TIME,
    params: {
      start_date: startDate,
      end_date: endDate,
    },
    enabled: !!startDate && !!endDate,
  });

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">Дашборд</h1>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 mb-6">
        <Card>
          <CardHeader>
            <CardTitle>Всего пользователей</CardTitle>
          </CardHeader>
          <CardContent>
            {isStatsPending ? (
              <p>Загрузка...</p>
            ) : (
              <p className="text-2xl font-bold">{stats?.total_users}</p>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Пользователи с покупками</CardTitle>
          </CardHeader>
          <CardContent>
            {isStatsPending ? (
              <p>Загрузка...</p>
            ) : (
              <p className="text-2xl font-bold">
                {stats?.users_with_purchases}
              </p>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Доступно товаров</CardTitle>
          </CardHeader>
          <CardContent>
            {isStatsPending ? (
              <p>Загрузка...</p>
            ) : (
              <p className="text-2xl font-bold">{stats?.available_products}</p>
            )}
          </CardContent>
        </Card>
      </div>

      <div>
        <h2 className="text-xl font-bold mb-4">Продажи за период</h2>
        <div className="flex gap-4 mb-4 items-center">
          <Input
            type="date"
            value={startDate.split("T")[0]}
            onChange={(e) => setStartDate(`${e.target.value}T00:00:00.000Z`)}
            className="max-w-sm"
          />
          <Input
            type="date"
            value={endDate.split("T")[0]}
            onChange={(e) => setEndDate(`${e.target.value}T00:00:00.000Z`)}
            className="max-w-sm"
          />
        </div>

        {isSalesPending && <p>Загрузка...</p>}

        {sales && !isSalesPending && (
          <div className="grid gap-4 md:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle>Продано товаров</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-2xl font-bold">{sales.products_sold}</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle>Общий доход</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-2xl font-bold">
                  {sales.total_revenue.toFixed(2)} ₽
                </p>
              </CardContent>
            </Card>
          </div>
        )}
      </div>
    </div>
  );
}
