package models

import "time"

type TimeSeriesData struct {
    Date  time.Time `json:"date"`
    Value int64     `json:"value"`
}

type CategorySales struct {
	CategoryName string  `json:"category_name"`
	TotalSales   float64 `json:"total_sales"`
}

type DashboardOverview struct {
	TotalUsers         int64 `json:"total_users"`
	UsersWithPurchases int64 `json:"users_with_purchases"`
	AvailableProducts  int64 `json:"available_products"`
	TotalUsers30Days         StatWithTrend `json:"total_users_30_days"`
	UsersWithPurchases30Days StatWithTrend `json:"users_with_purchases_30_days"`
	ProductsSold30Days       StatWithTrend `json:"products_sold_30_days"`
}

type StatWithTrend struct {
	Value    int64   `json:"value"`
	Trend    float64 `json:"trend"`
}