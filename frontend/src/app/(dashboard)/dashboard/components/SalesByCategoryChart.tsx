'use client';

import * as React from 'react';
import { PieChart } from '@mui/x-charts/PieChart';
import { useDrawingArea } from '@mui/x-charts/hooks';
import { styled } from '@mui/material/styles';
import Typography from '@mui/material/Typography';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';

interface CategorySales {
  category_name: string;
  total_sales: number;
}

interface SalesByCategoryChartProps {
  data: CategorySales[];
}

const StyledText = styled('text')(({ theme }) => ({
  textAnchor: 'middle',
  dominantBaseline: 'central',
  fill: theme.palette.text.secondary,
}));

function PieCenterLabel({ children }: { children: React.ReactNode }) {
  const { width, height, left, top } = useDrawingArea();
  return (
    <StyledText x={left + width / 2} y={top + height / 2}>
      {children}
    </StyledText>
  );
}

export default function SalesByCategoryChart({ data }: SalesByCategoryChartProps) {
  const chartData = data.map((item) => ({ label: item.category_name, value: item.total_sales }));

  return (
    <Card variant="outlined" sx={{ display: 'flex', flexDirection: 'column', gap: '8px', flexGrow: 1 }}>
      <CardContent>
        <Typography component="h2" variant="subtitle2">
          Продажи по категориям
        </Typography>
        <PieChart
          series={[
            {
              data: chartData,
              innerRadius: 75,
              outerRadius: 100,
              paddingAngle: 0,
              highlightScope: { faded: 'global', highlighted: 'item' },
            },
          ]}
          height={260}
          hideLegend
        >
          <PieCenterLabel>Продажи</PieCenterLabel>
        </PieChart>
      </CardContent>
    </Card>
  );
}
