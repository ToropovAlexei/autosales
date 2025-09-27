package models

type Product struct {
	ID                     uint     `gorm:"primaryKey" json:"id"`
	Name                   string   `gorm:"index" json:"name"`
	Price                  float64  `json:"price"`
	CategoryID             uint     `json:"category_id"`
	Category               Category `gorm:"foreignKey:CategoryID" json:"category"`
	Type                   string   `gorm:"default:'item'" json:"type"`
	SubscriptionPeriodDays int      `gorm:"default:0" json:"subscription_period_days"`
	Details                string   `gorm:"type:jsonb" json:"details"`
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
