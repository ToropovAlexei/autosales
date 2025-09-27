package models

import (
	"time"

	"gorm.io/datatypes"
)

type UserSubscription struct {
	ID            uint           `gorm:"primaryKey" json:"id"`
	BotUserID     uint           `gorm:"index" json:"bot_user_id"`
	BotUser       BotUser        `gorm:"foreignKey:BotUserID" json:"bot_user"`
	ProductID     uint           `gorm:"index" json:"product_id"`
	Product       Product        `gorm:"foreignKey:ProductID" json:"Product"`
	OrderID       uint           `gorm:"index" json:"order_id"`
	Order         Order          `gorm:"foreignKey:OrderID" json:"order"`
	ExpiresAt     time.Time      `gorm:"index" json:"expires_at"`
	IsActive      bool           `gorm:"default:true" json:"is_active"`
	ProvisionedID string         `gorm:"index" json:"provisioned_id"`
	Details       datatypes.JSON `gorm:"type:jsonb" json:"details"`
	CreatedAt     time.Time      `json:"created_at"`
	UpdatedAt     time.Time      `json:"updated_at"`
}
