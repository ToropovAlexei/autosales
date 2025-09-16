package responses

type BotUserResponse struct {
	ID               uint    `json:"id"`
	TelegramID       int64   `json:"telegram_id"`
	IsDeleted        bool    `json:"is_deleted"`
	HasPassedCaptcha bool    `json:"has_passed_captcha"`
	Balance          float64 `json:"balance"`
}
