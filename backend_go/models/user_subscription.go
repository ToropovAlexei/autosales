package models

import (
	"time"

	"gorm.io/datatypes"
)

type UserSubscription struct {
	ID          uint           `gorm:"primaryKey"`
	BotUserID   uint           `gorm:"index"`
	BotUser     BotUser        `gorm:"foreignKey:BotUserID"`
	ProductID   uint           `gorm:"index"`
	Product     Product        `gorm:"foreignKey:ProductID"`
	OrderID     uint           `gorm:"index"`
	Order       Order          `gorm:"foreignKey:OrderID"`
	ExpiresAt   time.Time      `gorm:"index"`
	IsActive    bool           `gorm:"default:true"`
	ProvisionedID string         `gorm:"index"`
	Details     datatypes.JSON `gorm:"type:jsonb"`
	CreatedAt   time.Time
	UpdatedAt   time.Time
}
