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
	SendNotification(botName string, telegramID int64, message string, messageToEdit *int64, messageToDelete *int64, inlineKeyboard [][]InlineKeyboardButton, stateToSet *string, stateData map[string]interface{}) error
	SendSuccessfulPaymentNotification(botName string, telegramID int64, amount float64, bonus int, messageToDelete *int64) error
	SendUnfinishedPaymentNotification(botName string, telegramID int64, amount float64, messageToDelete *int64) error
	SendCheckRequestNotification(botName string, telegramID int64, orderID string) error
	SendPaymentAppealNotification(botName string, telegramID int64, orderID string) error
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
	StateToSet      *string                  `json:"state_to_set,omitempty"`
	StateData       map[string]interface{}   `json:"state_data,omitempty"`
}

func (s *webhookService) SendNotification(botName string, telegramID int64, message string, messageToEdit *int64, messageToDelete *int64, inlineKeyboard [][]InlineKeyboardButton, stateToSet *string, stateData map[string]interface{}) error {
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
		StateToSet:      stateToSet,
		StateData:       stateData,
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

	return s.SendNotification(botName, telegramID, message, nil, messageToDelete, keyboard, nil, nil)
}

func (s *webhookService) SendUnfinishedPaymentNotification(botName string, telegramID int64, amount float64, messageToDelete *int64) error {
	message := fmt.Sprintf("⚠️ Не завершен платеж на сумму %.2f RUB", amount)
	return s.SendNotification(botName, telegramID, message, nil, messageToDelete, nil, nil, nil)
}

func (s *webhookService) SendCheckRequestNotification(botName string, telegramID int64, orderID string) error {
	message := "Не видим Ваш платеж. Пожалуйста, загрузите чек (в формате jpg или pdf) на сайт https://dropmefiles.com/ и отправьте нам полученную ссылку."
	stateToSet := "payment_awaiting_receipt_link"
	stateData := map[string]interface{}{"order_id": orderID}
	return s.SendNotification(botName, telegramID, message, nil, nil, nil, &stateToSet, stateData)
}

func (s *webhookService) SendPaymentAppealNotification(botName string, telegramID int64, orderID string) error {
	// TODO: Get operator contact from settings/config
	operatorContact := "@operator_contact_placeholder"
	message := fmt.Sprintf("Мы не смогли увидеть Ваш платеж. Пожалуйста свяжитесь с оператором: %s", operatorContact)
	return s.SendNotification(botName, telegramID, message, nil, nil, nil, nil, nil)
}
