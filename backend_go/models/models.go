package models

import (
	"time"
)

type UserRole string

const (
	Admin  UserRole = "admin"
	Seller UserRole = "seller"
)

type User struct {
	ID                      uint      `gorm:"primaryKey"`
	Email                   string    `gorm:"uniqueIndex"`
	HashedPassword          string
	IsActive                bool      `gorm:"default:true"`
	Role                    UserRole  `gorm:"default:seller;not null"`
	ReferralProgramEnabled  bool      `gorm:"default:false"`
	ReferralPercentage      float64   `gorm:"default:0.0"`
}

type Category struct {
	ID       uint      `gorm:"primaryKey"`
	Name     string    `gorm:"index"`
	Products []Product `gorm:"foreignKey:CategoryID"`
}

type Product struct {
	ID         uint      `gorm:"primaryKey"`
	Name       string    `gorm:"index"`
	Price      float64
	CategoryID uint
	Category   Category `gorm:"foreignKey:CategoryID"`
}

type BotUser struct {
	ID               uint   `gorm:"primaryKey"`
	TelegramID       int64  `gorm:"uniqueIndex"`
	IsDeleted        bool   `gorm:"default:false"`
	HasPassedCaptcha bool   `gorm:"default:false"`
}

type TransactionType string

const (
	Deposit  TransactionType = "deposit"
	Purchase TransactionType = "purchase"
)

type Transaction struct {
	ID          uint            `gorm:"primaryKey"`
	UserID      uint
	OrderID     *uint
	Type        TransactionType `gorm:"not null"`
	Amount      float64         `gorm:"not null"`
	CreatedAt   time.Time       `gorm:"not null"`
	Description string
}

type Order struct {
	ID        uint      `gorm:"primaryKey"`
	UserID    uint
	ProductID uint
	Quantity  int       `gorm:"default:1"`
	Amount    float64
	Status    string
	CreatedAt time.Time `gorm:"not null;default:now()"`
}

type StockMovementType string

const (
	Initial StockMovementType = "initial"
	Sale    StockMovementType = "sale"
	Restock StockMovementType = "restock"
	Return  StockMovementType = "return"
)

type StockMovement struct {
	ID          uint              `gorm:"primaryKey"`
	OrderID     *uint
	ProductID   uint
	Type        StockMovementType `gorm:"not null"`
	Quantity    int               `gorm:"not null"`
	CreatedAt   time.Time         `gorm:"not null"`
	Description string
}

type ReferralBot struct {
	ID        uint      `gorm:"primaryKey"`
	OwnerID   uint
	SellerID  uint
	BotToken  string    `gorm:"unique"`
	IsActive  bool      `gorm:"default:true"`
	CreatedAt time.Time `gorm:"not null;default:now()"`
}

type RefTransaction struct {
	ID         uint      `gorm:"primaryKey"`
	RefOwnerID uint
	SellerID   uint
	OrderID    uint
	Amount     float64   `gorm:"not null"`
	RefShare   float64   `gorm:"not null"`
	CreatedAt  time.Time `gorm:"not null;default:now()"`
}
