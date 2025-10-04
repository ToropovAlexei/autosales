'use client';

import { useOne } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { InputDate } from "@/components";
import { FormProvider, useForm, useWatch } from "react-hook-form";
import dayjs from "dayjs";
import { Card, CardContent, CardHeader, Typography } from "@mui/material";
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import classes from './styles.module.css';
import clsx from 'clsx';

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
    <LocalizationProvider dateAdapter={AdapterDayjs}>
      <div className={classes.page}>
        <Typography variant="h4" gutterBottom>
          Дашборд
        </Typography>

        <div className={classes.grid}>
          <Card>
            <CardHeader title="Всего пользователей" />
            <CardContent>
              {isStatsPending ? (
                <p>Загрузка...</p>
              ) : (
                <Typography variant="h5">{stats?.total_users}</Typography>
              )}
            </CardContent>
          </Card>
          <Card>
            <CardHeader title="Пользователи с покупками" />
            <CardContent>
              {isStatsPending ? (
                <p>Загрузка...</p>
              ) : (
                <Typography variant="h5">{stats?.users_with_purchases}</Typography>
              )}
            </CardContent>
          </Card>
          <Card>
            <CardHeader title="Доступно товаров" />
            <CardContent>
              {isStatsPending ? (
                <p>Загрузка...</p>
              ) : (
                <Typography variant="h5">{stats?.available_products}</Typography>
              )}
            </CardContent>
          </Card>
        </div>

        <div>
          <Typography variant="h5" gutterBottom>
            Продажи за период
          </Typography>
          <FormProvider {...form}>
            <div className={classes.datePickerContainer}>
              <InputDate name="start_date" />
              <InputDate name="end_date" />
            </div>
          </FormProvider>

          {isSalesPending && <p>Загрузка...</p>}

          {sales && !isSalesPending && (
            <div className={clsx(classes.grid, classes['two-columns'])}>
              <Card>
                <CardHeader title="Продано товаров" />
                <CardContent>
                  <Typography variant="h5">{sales.products_sold}</Typography>
                </CardContent>
              </Card>
              <Card>
                <CardHeader title="Общий доход" />
                <CardContent>
                  <Typography variant="h5">{sales.total_revenue.toFixed(2)} ₽</Typography>
                </CardContent>
              </Card>
            </div>
          )}
        </div>
      </div>
    </LocalizationProvider>
  );
}