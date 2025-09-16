package responses

import "time"

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
