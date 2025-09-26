package models

type Product struct {
	ID         uint   `gorm:"primaryKey"`
	Name       string `gorm:"index"`
	Price                  float64
	CategoryID             uint
	Category               Category `gorm:"foreignKey:CategoryID"`
	Type                   string   `gorm:"default:'item'"`
	SubscriptionPeriodDays int      `gorm:"default:0"`
}

type ProductResponse struct {
	ID         uint    `json:"id"`
	Name       string  `json:"name"`
	Price      float64 `json:"price"`
	CategoryID uint    `json:"category_id"`
	Stock                  int      `json:"stock"`
	Type                   string   `json:"type"`
	SubscriptionPeriodDays int      `json:"subscription_period_days"`
	Provider               string   `json:"provider,omitempty"`
	ExternalID             string   `json:"external_id,omitempty"`
}
