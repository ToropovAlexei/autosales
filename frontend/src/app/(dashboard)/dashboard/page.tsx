"use client";

import { useOne } from "@/hooks";
import { ENDPOINTS } from "@/constants";
import { InputDate } from "@/components";
import { FormProvider, useForm, useWatch } from "react-hook-form";
import dayjs from "dayjs";
import { Typography } from "@mui/material";
import { LocalizationProvider } from "@mui/x-date-pickers/LocalizationProvider";
import { AdapterDayjs } from "@mui/x-date-pickers/AdapterDayjs";
import { StatCard } from "@/components/StatCard";
import { SessionsChart } from "@/components/SessionsChart";
import { PageViewsBarChart } from "@/components/PageViewsBarChart";
import { TopProductsTable } from "./components/TopProductsTable";
import { SalesByCategoryChart } from "./components/SalesByCategoryChart";
import classes from "./styles.module.css";
import clsx from "clsx";
import { IProduct } from "@/types";
import { PageLayout } from "@/components/PageLayout";

interface TimeSeriesDataPoint {
  date: string;
  value: number;
}

interface TimeSeriesDashboardData {
  sales: {
    products_sold: number;
    total_revenue: number;
  };
  sales_chart: TimeSeriesDataPoint[];
  users_chart: TimeSeriesDataPoint[];
  revenue_chart: TimeSeriesDataPoint[];
  users_with_purchases_chart: TimeSeriesDataPoint[];
}

interface StatWithTrend {
  value: number;
  trend: number;
}

interface DashboardOverview {
  total_users: number;
  users_with_purchases: number;
  available_products: number;
  total_users_30_days: StatWithTrend;
  users_with_purchases_30_days: StatWithTrend;
  products_sold_30_days: StatWithTrend;
}

interface CategorySales {
  category_name: string;
  total_sales: number;
}

export default function DashboardPage() {
  const { data: overview, isPending: isOverviewPending } =
    useOne<DashboardOverview>({
      endpoint: ENDPOINTS.DASHBOARD_STATS,
    });

  const { data: topProducts, isPending: isTopProductsPending } = useOne<
    IProduct[]
  >({
    endpoint: ENDPOINTS.DASHBOARD_TOP_PRODUCTS,
  });

  const { data: salesByCategory, isPending: isSalesByCategoryPending } = useOne<
    CategorySales[]
  >({
    endpoint: ENDPOINTS.DASHBOARD_SALES_BY_CATEGORY,
  });

  const form = useForm<{ start_date: string; end_date: string }>({
    defaultValues: {
      start_date: dayjs().subtract(30, "day").startOf("day").toISOString(),
      end_date: dayjs().endOf("day").toISOString(),
    },
  });
  const { start_date, end_date } = useWatch({ control: form.control });

  const { data: timeSeriesData, isPending: isTimeSeriesPending } =
    useOne<TimeSeriesDashboardData>({
      endpoint: ENDPOINTS.DASHBOARD_TIME_SERIES,
      params: { start_date, end_date },
      enabled: !!start_date && !!end_date,
    });

  const getTrend = (trend: number) => {
    if (trend > 0) return "up";
    if (trend < 0) return "down";
    return "neutral";
  };

  return (
    <LocalizationProvider dateAdapter={AdapterDayjs}>
      <PageLayout title="Дашборд">
        <div className={classes.grid}>
          {isOverviewPending ? (
            <p>Загрузка...</p>
          ) : (
            <StatCard
              title="Всего пользователей"
              value={overview?.total_users.toString() || "0"}
              interval="Все время"
              trend="neutral"
              data={timeSeriesData?.users_chart?.map((d) => d.value) || []}
              xAxisData={
                timeSeriesData?.users_chart?.map((d) =>
                  dayjs(d.date).format("DD.MM")
                ) || []
              }
            />
          )}
          {isOverviewPending ? (
            <p>Загрузка...</p>
          ) : (
            <StatCard
              title="Пользователи с покупками"
              value={overview?.users_with_purchases.toString() || "0"}
              interval="Все время"
              trend="neutral"
              data={
                timeSeriesData?.users_with_purchases_chart?.map(
                  (d) => d.value
                ) || []
              }
              xAxisData={
                timeSeriesData?.users_with_purchases_chart?.map((d) =>
                  dayjs(d.date).format("DD.MM")
                ) || []
              }
            />
          )}
          {isOverviewPending ? (
            <p>Загрузка...</p>
          ) : (
            <StatCard
              title="Доступно товаров"
              value={overview?.available_products.toString() || "0"}
              interval="Все время"
              trend="neutral"
              data={[]}
              xAxisData={[]}
            />
          )}
        </div>

        <div className={classes.grid} style={{ marginTop: "24px" }}>
          {isOverviewPending ? (
            <p>Загрузка...</p>
          ) : (
            <StatCard
              title="Новые пользователи"
              value={overview?.total_users_30_days.value.toString() || "0"}
              interval={`с ${dayjs()
                .subtract(30, "day")
                .startOf("day")
                .format("DD.MM.YYYY")} по ${dayjs()
                .endOf("day")
                .format("DD.MM.YYYY")}`}
              trend={getTrend(overview?.total_users_30_days.trend || 0)}
              data={timeSeriesData?.users_chart?.map((d) => d.value) || []}
              xAxisData={
                timeSeriesData?.users_chart?.map((d) =>
                  dayjs(d.date).format("DD.MM")
                ) || []
              }
            />
          )}
          {isOverviewPending ? (
            <p>Загрузка...</p>
          ) : (
            <StatCard
              title="Пользователи с покупками (30 дней)"
              value={
                overview?.users_with_purchases_30_days.value.toString() || "0"
              }
              interval={`с ${dayjs()
                .subtract(30, "day")
                .startOf("day")
                .format("DD.MM.YYYY")} по ${dayjs()
                .endOf("day")
                .format("DD.MM.YYYY")}`}
              trend={getTrend(
                overview?.users_with_purchases_30_days.trend || 0
              )}
              data={
                timeSeriesData?.users_with_purchases_chart?.map(
                  (d) => d.value
                ) || []
              }
              xAxisData={
                timeSeriesData?.users_with_purchases_chart?.map((d) =>
                  dayjs(d.date).format("DD.MM")
                ) || []
              }
            />
          )}
          {isOverviewPending ? (
            <p>Загрузка...</p>
          ) : (
            <StatCard
              title="Продано товаров (30 дней)"
              value={overview?.products_sold_30_days.value.toString() || "0"}
              interval={`с ${dayjs()
                .subtract(30, "day")
                .startOf("day")
                .format("DD.MM.YYYY")} по ${dayjs()
                .endOf("day")
                .format("DD.MM.YYYY")}`}
              trend={getTrend(overview?.products_sold_30_days.trend || 0)}
              data={timeSeriesData?.sales_chart?.map((d) => d.value) || []}
              xAxisData={
                timeSeriesData?.sales_chart?.map((d) =>
                  dayjs(d.date).format("DD.MM")
                ) || []
              }
            />
          )}
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

          {isTimeSeriesPending && <p>Загрузка...</p>}

          {timeSeriesData && !isTimeSeriesPending && (
            <div className={clsx(classes.grid, classes["two-columns"])}>
              <StatCard
                title="Продано товаров"
                value={timeSeriesData.sales.products_sold.toString()}
                interval={`с ${dayjs(start_date).format(
                  "DD.MM.YYYY"
                )} по ${dayjs(end_date).format("DD.MM.YYYY")}`}
                trend="neutral"
                data={timeSeriesData?.sales_chart?.map((d) => d.value) || []}
                xAxisData={
                  timeSeriesData?.sales_chart?.map((d) =>
                    dayjs(d.date).format("DD.MM")
                  ) || []
                }
              />
              <StatCard
                title="Общий доход"
                value={`${timeSeriesData.sales.total_revenue.toFixed(2)} ₽`}
                interval={`с ${dayjs(start_date).format(
                  "DD.MM.YYYY"
                )} по ${dayjs(end_date).format("DD.MM.YYYY")}`}
                trend="neutral"
                data={timeSeriesData?.revenue_chart?.map((d) => d.value) || []}
                xAxisData={
                  timeSeriesData?.revenue_chart?.map((d) =>
                    dayjs(d.date).format("DD.MM")
                  ) || []
                }
              />
            </div>
          )}
        </div>

        <div className={classes.grid} style={{ marginTop: "24px" }}>
          {isTimeSeriesPending ? (
            <p>Загрузка...</p>
          ) : (
            <SessionsChart
              title="Продажи"
              subtitle={`за период с ${dayjs(start_date).format(
                "DD.MM.YYYY"
              )} по ${dayjs(end_date).format("DD.MM.YYYY")}`}
              data={timeSeriesData?.sales_chart?.map((d) => d.value) || []}
              labels={
                timeSeriesData?.sales_chart?.map((d) =>
                  dayjs(d.date).format("DD.MM")
                ) || []
              }
            />
          )}
          {isTimeSeriesPending ? (
            <p>Загрузка...</p>
          ) : (
            <PageViewsBarChart
              title="Новые пользователи"
              subtitle={`за период с ${dayjs(start_date).format(
                "DD.MM.YYYY"
              )} по ${dayjs(end_date).format("DD.MM.YYYY")}`}
              data={timeSeriesData?.users_chart?.map((d) => d.value) || []}
              labels={
                timeSeriesData?.users_chart?.map((d) =>
                  dayjs(d.date).format("DD.MM")
                ) || []
              }
            />
          )}
        </div>

        <div className={classes.grid} style={{ marginTop: "24px" }}>
          {isTopProductsPending ? (
            <p>Загрузка...</p>
          ) : (
            <TopProductsTable products={topProducts || []} />
          )}
          {isSalesByCategoryPending ? (
            <p>Загрузка...</p>
          ) : (
            <SalesByCategoryChart data={salesByCategory || []} />
          )}
        </div>
      </PageLayout>
    </LocalizationProvider>
  );
}
