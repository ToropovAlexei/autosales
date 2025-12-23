package models

type BroadcastPayload struct {
	Text     *string           `json:"text"`
	ImageID  *string           `json:"image_id"`
	Filters  BroadcastFilters `json:"filters"`
}

type RedisBroadcastMessage struct {
	TelegramID int64   `json:"telegram_id"`
	Text       *string `json:"text"`
	ImageID    *string `json:"image_id"`
}
