'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import api from '@/lib/api';

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
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [sales, setSales] = useState<SalesOverTime | null>(null);
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');
  const [loading, setLoading] = useState(true);
  const [salesLoading, setSalesLoading] = useState(false);

  useEffect(() => {
    const fetchStats = async () => {
      try {
        setLoading(true);
        const data = await api.getDashboardStats();
        setStats(data);
      } catch (error) {
        console.error('Failed to fetch dashboard stats', error);
      } finally {
        setLoading(false);
      }
    };

    fetchStats();
  }, []);

  const handleFetchSales = async () => {
    if (!startDate || !endDate) {
      alert('Please select both start and end dates.');
      return;
    }
    try {
      setSalesLoading(true);
      const data = await api.getSalesOverTime(startDate, endDate);
      setSales(data);
    } catch (error) {
      console.error('Failed to fetch sales data', error);
    } finally {
      setSalesLoading(false);
    }
  };

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">Dashboard</h1>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 mb-6">
        <Card>
          <CardHeader>
            <CardTitle>Total Users</CardTitle>
          </CardHeader>
          <CardContent>
            {loading ? <p>Loading...</p> : <p className="text-2xl font-bold">{stats?.total_users}</p>}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Users with Purchases</CardTitle>
          </CardHeader>
          <CardContent>
            {loading ? <p>Loading...</p> : <p className="text-2xl font-bold">{stats?.users_with_purchases}</p>}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Available Products</CardTitle>
          </CardHeader>
          <CardContent>
            {loading ? <p>Loading...</p> : <p className="text-2xl font-bold">{stats?.available_products}</p>}
          </CardContent>
        </Card>
      </div>

      <div>
        <h2 className="text-xl font-bold mb-4">Sales Over Time</h2>
        <div className="flex gap-4 mb-4 items-center">
          <Input
            type="date"
            value={startDate}
            onChange={(e) => setStartDate(e.target.value)}
            className="max-w-sm"
          />
          <Input
            type="date"
            value={endDate}
            onChange={(e) => setEndDate(e.target.value)}
            className="max-w-sm"
          />
          <Button onClick={handleFetchSales} disabled={salesLoading}>
            {salesLoading ? 'Loading...' : 'Get Sales'}
          </Button>
        </div>

        {sales && (
          <div className="grid gap-4 md:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle>Products Sold</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-2xl font-bold">{sales.products_sold}</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle>Total Revenue</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-2xl font-bold">${sales.total_revenue.toFixed(2)}</p>
              </CardContent>
            </Card>
          </div>
        )}
      </div>
    </div>
  );
}
