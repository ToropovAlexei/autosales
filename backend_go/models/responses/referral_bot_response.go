package responses

import "time"

type ReferralBotResponse struct {
	ID        uint      `json:"id"`
	OwnerID   uint      `json:"owner_id"`
	SellerID  uint      `json:"seller_id"`
	BotToken  string    `json:"bot_token"`
	IsActive  bool      `json:"is_active"`
	CreatedAt time.Time `json:"created_at"`
}
