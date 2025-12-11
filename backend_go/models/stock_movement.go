package models

import (
	"time"
)

type StockMovementType string

const (
	Initial    StockMovementType = "initial"
	Sale       StockMovementType = "sale"
	Restock    StockMovementType = "restock"
	Return     StockMovementType = "return"
	Adjustment StockMovementType = "adjustment"
)

type StockMovement struct {
	ID          uint `gorm:"primaryKey"`
	OrderID     *uint
	ProductID   uint              `gorm:"column:product_id"`
	Product     Product           `gorm:"foreignKey:ProductID"`
	Type        StockMovementType `gorm:"not null"`
	Quantity    int               `gorm:"not null"`
	CreatedAt   time.Time         `gorm:"not null"`
	Description string
}

type StockMovementResponse struct {
	ID          uint              `json:"id"`
	ProductID   uint              `json:"product_id"`
	ProductName string            `json:"product_name"`
	Type        StockMovementType `json:"type"`
	Quantity    int               `json:"quantity"`
	CreatedAt   time.Time         `json:"created_at"`
	Description string            `json:"description"`
	OrderID     *uint             `json:"order_id"`
}
