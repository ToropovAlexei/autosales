package models

type UpdateBotUserStatusPayload struct {
	BotIsBlockedByUser *bool `json:"bot_is_blocked_by_user"`
	IsBlocked          *bool `json:"is_blocked"`
}
