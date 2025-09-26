package models

import "time"

type UserSubscription struct {
	ID        uint      `gorm:"primaryKey"`
	UserID    uint      `gorm:"index"`
	User      User      `gorm:"foreignKey:UserID"`
	ProductID uint      `gorm:"index"`
	Product   Product   `gorm:"foreignKey:ProductID"`
	OrderID   uint      `gorm:"index"`
	Order     Order     `gorm:"foreignKey:OrderID"`
	ExpiresAt time.Time `gorm:"index"`
	IsActive  bool      `gorm:"default:true"`
	CreatedAt time.Time
	UpdatedAt time.Time
}
