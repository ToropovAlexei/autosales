package models

type StoreBalance struct {
	ID             uint    `gorm:"primaryKey"`
	CurrentBalance float64 `gorm:"not null"`
}

type StoreBalanceResponse struct {
	CurrentBalance float64 `json:"current_balance"`
}
