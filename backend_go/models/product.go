package models

import (
	"database/sql"

	"github.com/google/uuid"
)

type Product struct {
	ID                     uint           `gorm:"primaryKey" json:"id"`
	Name                   string         `gorm:"index" json:"name"`
	Price                  float64        `json:"price"`
	CategoryID             uint           `json:"category_id"`
	Category               Category       `gorm:"foreignKey:CategoryID" json:"category"`
	ImageID                *uuid.UUID     `gorm:"type:uuid" json:"image_id"`
	Image                  *Image         `gorm:"foreignKey:ImageID" json:"image"`
	Type                   string         `gorm:"default:'item'" json:"type"`
	SubscriptionPeriodDays int            `gorm:"default:0" json:"subscription_period_days"`
	Details                sql.NullString `gorm:"type:jsonb" json:"details"`
	Visible                bool           `gorm:"default:true" json:"visible"`
	FulfillmentType        string         `gorm:"default:'none'" json:"fulfillment_type"` // none, text, image_url
	FulfillmentContent     string         `gorm:"type:text" json:"fulfillment_content"`   // The content to be delivered
}

type ProductResponse struct {
	ID                     uint    `json:"id"`
	Name                   string  `json:"name"`
	Price                  float64 `json:"price"`
	CategoryID             uint    `json:"category_id"`
	ImageUrl               string  `json:"image_url"`
	Stock                  int     `json:"stock"`
	Type                   string  `json:"type"`
	SubscriptionPeriodDays int     `json:"subscription_period_days"`
	Provider               string  `json:"provider,omitempty"`
	ExternalID             string  `json:"external_id,omitempty"`
	Visible                bool    `json:"visible"`
	FulfillmentType        string  `json:"fulfillment_type"`
	FulfillmentContent     string  `json:"fulfillment_content"`
}
