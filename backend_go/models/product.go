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
	FulfillmentText        sql.NullString `gorm:"type:text" json:"fulfillment_text"`     // The text content to be delivered
	FulfillmentImageID     *uuid.UUID     `gorm:"type:uuid" json:"fulfillment_image_id"` // The image ID to be delivered
	FulfillmentImage       *Image         `gorm:"foreignKey:FulfillmentImageID" json:"fulfillment_image"`
	ProviderName           *string        `gorm:"index" json:"provider_name"`
	ExternalID             *string        `gorm:"index" json:"external_id"`
}

type ProductResponse struct {
	ID                     uint    `json:"id"`
	Name                   string  `json:"name"`
	BasePrice              float64 `json:"base_price"`
	Price                  float64 `json:"price"`
	CategoryID             uint    `json:"category_id"`
	ImageID                string  `json:"image_id,omitempty"`
	Stock                  int     `json:"stock"`
	Type                   string  `json:"type"`
	SubscriptionPeriodDays int     `json:"subscription_period_days"`
	Provider               string  `json:"provider,omitempty"`
	ExternalID             string  `json:"external_id,omitempty"`
	Visible                bool    `json:"visible"`
	FulfillmentText        string  `json:"fulfillment_text,omitempty"`
	FulfillmentImageID     string  `json:"fulfillment_image_id,omitempty"`
}
