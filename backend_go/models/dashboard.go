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
