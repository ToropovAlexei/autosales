package models

import "time"

type CategoryResponse struct {
	ID   uint   `json:"id"`
	Name string `json:"name"`
}

type ProductResponse struct {
	ID         uint    `json:"id"`
	Name       string  `json:"name"`
	Price      float64 `json:"price"`
	CategoryID uint    `json:"category_id"`
	Stock      int     `json:"stock"`
}

type UserResponse struct {
	ID                     uint      `json:"id"`
	Email                  string    `json:"email"`
	IsActive               bool      `json:"is_active"`
	Role                   UserRole  `json:"role"`
	ReferralProgramEnabled bool      `json:"referral_program_enabled"`
	ReferralPercentage     float64   `json:"referral_percentage"`
}

type BotUserResponse struct {
	ID               uint    `json:"id"`
	TelegramID       int64   `json:"telegram_id"`
	IsDeleted        bool    `json:"is_deleted"`
	HasPassedCaptcha bool    `json:"has_passed_captcha"`
	Balance          float64 `json:"balance"`
}

type TransactionResponse struct {
	ID          uint            `json:"id"`
	UserID      uint            `json:"user_id"`
	OrderID     *uint           `json:"order_id"`
	Type        TransactionType `json:"type"`
	Amount      float64         `json:"amount"`
	CreatedAt   time.Time       `json:"created_at"`
	Description string          `json:"description"`
}

type OrderResponse struct {
	ID             uint      `json:"id"`
	UserID         uint      `json:"user_id"`
	ProductID      uint      `json:"product_id"`
	Quantity       int       `json:"quantity"`
	Amount         float64   `json:"amount"`
	Status         string    `json:"status"`
	CreatedAt      time.Time `json:"created_at"`
	UserTelegramID int64     `json:"user_telegram_id"`
	ProductName    string    `json:"product_name"`
}

type OrderSlimResponse struct {
	ID        uint      `json:"id"`
	UserID    uint      `json:"user_id"`
	ProductID uint      `json:"product_id"`
	Quantity  int       `json:"quantity"`
	Amount    float64   `json:"amount"`
	Status    string    `json:"status"`
	CreatedAt time.Time `json:"created_at"`
}

type StockMovementResponse struct {
	ID          uint            `json:"id"`
	ProductID   uint            `json:"product_id"`
	Type        StockMovementType `json:"type"`
	Quantity    int             `json:"quantity"`
	CreatedAt   time.Time       `json:"created_at"`
	Description string          `json:"description"`
	OrderID     *uint           `json:"order_id"`
}

type ReferralBotResponse struct {
	ID        uint      `json:"id"`
	OwnerID   uint      `json:"owner_id"`
	SellerID  uint      `json:"seller_id"`
	BotToken  string    `json:"bot_token"`
	IsActive  bool      `json:"is_active"`
	CreatedAt time.Time `json:"created_at"`
}

type ReferralBotAdminInfo struct {
	ID              uint      `json:"id"`
	OwnerID         uint      `json:"owner_id"`
	SellerID        uint      `json:"seller_id"`
	BotToken        string    `json:"bot_token"`
	IsActive        bool      `json:"is_active"`
	CreatedAt       time.Time `json:"created_at"`
	OwnerTelegramID int64     `json:"owner_telegram_id"`
	Turnover        float64   `json:"turnover"`
	Accruals        float64   `json:"accruals"`
}
