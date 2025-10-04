"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useOne } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { InputDate } from "@/components";
import { FormProvider, useForm, useWatch } from "react-hook-form";
import dayjs from "dayjs";

interface DashboardStats {
  total_users: number;
  users_with_purchases: number;
  available_products: number;
}

interface SalesOverTime {
  products_sold: number;
  total_revenue: number;
}

export default function DashboardPage() {
  const { data: stats, isPending: isStatsPending } = useOne<DashboardStats>({
    endpoint: ENDPOINTS.DASHBOARD_STATS,
  });

  const form = useForm<{ start_date: string; end_date: string }>({
    defaultValues: {
      start_date: dayjs().subtract(7, "day").startOf("day").toISOString(),
      end_date: dayjs().endOf("day").toISOString(),
    },
  });
  const { start_date, end_date } = useWatch({ control: form.control });
  const { data: sales, isPending: isSalesPending } = useOne<SalesOverTime>({
    endpoint: ENDPOINTS.SALES_OVER_TIME,
    params: { start_date, end_date },
    enabled: !!start_date && !!end_date,
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
          <FormProvider {...form}>
            <InputDate name="start_date" />
            <InputDate name="end_date" />
          </FormProvider>
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
