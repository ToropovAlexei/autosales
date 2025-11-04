package services

import (
	"bytes"
	"encoding/json"
	"fmt"
	"frbktg/backend_go/config"
	"net/http"
)

type WebhookService interface {
	SendNotification(botName string, telegramID int64, message string, messageToEdit *int64, messageToDelete *int64) error
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
	BotName         string `json:"bot_name"`
	TelegramID      int64  `json:"telegram_id"`
	Message         string `json:"message"`
	MessageToEdit   *int64 `json:"message_to_edit,omitempty"`
	MessageToDelete *int64 `json:"message_to_delete,omitempty"`
}

func (s *webhookService) SendNotification(botName string, telegramID int64, message string, messageToEdit *int64, messageToDelete *int64) error {
	if s.dispatcherURL == "" {
		return nil
	}

	payload := notificationPayload{
		BotName:         botName,
		TelegramID:      telegramID,
		Message:         message,
		MessageToEdit:   messageToEdit,
		MessageToDelete: messageToDelete,
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
