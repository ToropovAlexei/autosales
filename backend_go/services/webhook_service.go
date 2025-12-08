package services

import (
	"bytes"
	"encoding/json"
	"fmt"
	"frbktg/backend_go/config"
	"net/http"
)

type InlineKeyboardButton struct {
	Text         string `json:"text"`
	CallbackData string `json:"callback_data"`
}

type WebhookService interface {
	SendNotification(botName string, telegramID int64, message string, messageToEdit *int64, messageToDelete *int64, inlineKeyboard [][]InlineKeyboardButton) error
	SendSuccessfulPaymentNotification(botName string, telegramID int64, amount float64, bonus int, messageToDelete *int64) error
	SendUnfinishedPaymentNotification(telegramID int64, amount float64, messageToDelete *int64) error
}

type webhookService struct {
	dispatcherURL string
	serviceAPIKey string
	httpClient    *http.Client
}

func NewWebhookService(cfg *config.Config) WebhookService {
	return &webhookService{
		dispatcherURL: cfg.BotDispatcherWebhookURL,
		serviceAPIKey: cfg.ServiceAPIKey,
		httpClient:    &http.Client{},
	}
}

type notificationPayload struct {
	BotName         string                   `json:"bot_name"`
	TelegramID      int64                    `json:"telegram_id"`
	Message         string                   `json:"message"`
	MessageToEdit   *int64                   `json:"message_to_edit,omitempty"`
	MessageToDelete *int64                   `json:"message_to_delete,omitempty"`
	InlineKeyboard  [][]InlineKeyboardButton `json:"inline_keyboard,omitempty"`
}

func (s *webhookService) SendNotification(botName string, telegramID int64, message string, messageToEdit *int64, messageToDelete *int64, inlineKeyboard [][]InlineKeyboardButton) error {
	if s.dispatcherURL == "" {
		return nil
	}

	payload := notificationPayload{
		BotName:         botName,
		TelegramID:      telegramID,
		Message:         message,
		MessageToEdit:   messageToEdit,
		MessageToDelete: messageToDelete,
		InlineKeyboard:  inlineKeyboard,
	}

	payloadBytes, err := json.Marshal(payload)
	if err != nil {
		return fmt.Errorf("failed to marshal notification payload: %w", err)
	}

	req, err := http.NewRequest("POST", s.dispatcherURL, bytes.NewBuffer(payloadBytes))
	if err != nil {
		return fmt.Errorf("failed to create notification request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-API-KEY", s.serviceAPIKey)

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("failed to send notification webhook: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 300 {
		return fmt.Errorf("notification webhook failed with status code: %d", resp.StatusCode)
	}

	return nil
}

func (s *webhookService) SendSuccessfulPaymentNotification(botName string, telegramID int64, amount float64, bonus int, messageToDelete *int64) error {
	message := fmt.Sprintf("✅ Баланс пополнен на %.2f RUB", amount)
	if bonus > 0 {
		message += fmt.Sprintf(" (включая бонус %d%%)", bonus)
	}

	keyboard := [][]InlineKeyboardButton{
		{
			{Text: "⬅️ Главное меню", CallbackData: "main_menu"},
		},
	}

	return s.SendNotification(botName, telegramID, message, nil, messageToDelete, keyboard)
}

func (s *webhookService) SendUnfinishedPaymentNotification(telegramID int64, amount float64, messageToDelete *int64) error {
	message := fmt.Sprintf("⚠️ Не завершен платеж на сумму %.2f RUB", amount)
	return s.SendNotification("", telegramID, message, nil, messageToDelete, nil)
}
