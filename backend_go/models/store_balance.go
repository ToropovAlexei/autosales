package models

import "gorm.io/gorm"

type StoreBalance struct {
	gorm.Model
	CurrentBalance float64 `gorm:"not null;default:0"`
}

type StoreBalanceResponse struct {
	CurrentBalance float64 `json:"current_balance"`
}
