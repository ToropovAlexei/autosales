package services

import (
	"errors"
	"fmt"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/config"
	"frbktg/backend_go/gateways"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"log/slog"
	"net/http"
	"strconv"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

// CreateInvoiceResponse is a custom response that includes both external gateway data and our internal OrderID
type CreateInvoiceResponse struct {
	PayURL           *string     `json:"pay_url,omitempty"`
	GatewayInvoiceID string      `json:"gateway_invoice_id"`
	OrderID          string      `json:"order_id"` // Our internal ID
	Details          interface{} `json:"details,omitempty"`
}

type PaymentService interface {
	GetAvailableGateways() []gateways.PaymentGateway
	CreateInvoice(telegramID int64, gatewayName string, amount float64) (*CreateInvoiceResponse, error)
	SetInvoiceMessageID(orderID string, messageID int64) error
	HandleWebhook(gatewayName string, r *http.Request) error
	NotifyUnfinishedPayments() error
	PollPendingPayments() error
}

type paymentService struct {
	db              *gorm.DB
	registry        *gateways.ProviderRegistry
	invoiceRepo     repositories.PaymentInvoiceRepository
	transactionRepo repositories.TransactionRepository
	botUserRepo     repositories.BotUserRepository
	webhookService  WebhookService
	settingService  *SettingService
	config          config.Settings
}

func NewPaymentService(db *gorm.DB, registry *gateways.ProviderRegistry, invoiceRepo repositories.PaymentInvoiceRepository, transactionRepo repositories.TransactionRepository, botUserRepo repositories.BotUserRepository, webhookService WebhookService, settingService *SettingService, config config.Settings) PaymentService {
	return &paymentService{
		db:              db,
		registry:        registry,
		invoiceRepo:     invoiceRepo,
		transactionRepo: transactionRepo,
		botUserRepo:     botUserRepo,
		webhookService:  webhookService,
		settingService:  settingService,
		config:          config,
	}
}

func (s *paymentService) GetAvailableGateways() []gateways.PaymentGateway {
	return s.registry.GetAllProviders()
}

func (s *paymentService) CreateInvoice(telegramID int64, gatewayName string, amount float64) (*CreateInvoiceResponse, error) {
	if gatewayName == "" {
		settings, err := s.settingService.GetSettings()
		if err != nil {
			return nil, apperrors.New(http.StatusInternalServerError, "Failed to get settings for default gateway", err)
		}
		defaultGateway, ok := settings["DEFAULT_PAYMENT_GATEWAY"]
		if !ok || defaultGateway == "" {
			return nil, apperrors.New(http.StatusInternalServerError, "Default payment gateway is not configured", nil)
		}
		gatewayName = defaultGateway
		slog.Debug("no gateway specified, using default", "default_gateway", gatewayName)
	}

	gateway, err := s.registry.GetProvider(gatewayName)
	if err != nil {
		return nil, apperrors.New(http.StatusBadRequest, "Invalid payment gateway", err)
	}

	botUser, err := s.botUserRepo.FindByTelegramID(telegramID)
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, apperrors.New(http.StatusNotFound, "Bot user not found", err)
		}
		return nil, apperrors.New(http.StatusInternalServerError, "Failed to find bot user", err)
	}

	orderID := uuid.New().String()

	// --- Gateway Discount Logic ---
	allSettings, err := s.settingService.GetSettings()
	if err != nil {
		slog.Error("could not retrieve settings to check for gateway discount", "error", err)
	}

	discountKey := "GATEWAY_DISCOUNT_" + gatewayName
	discountPercentageStr, hasDiscount := allSettings[discountKey]

	invoiceAmount := amount
	if hasDiscount && err == nil {
		if discountPercentage, parseErr := strconv.ParseFloat(discountPercentageStr, 64); parseErr == nil && discountPercentage > 0 {
			invoiceAmount = amount * (1 - discountPercentage/100.0)
		}
	}
	// --- End Gateway Discount Logic ---

	invoiceReq := &gateways.InvoiceCreationRequest{
		Amount:  invoiceAmount,
		UserID:  botUser.ID,
		OrderID: orderID,
	}

	externalInvoice, err := gateway.CreateInvoice(invoiceReq)
	if err != nil {
		return nil, apperrors.New(http.StatusInternalServerError, "Failed to create external invoice", err)
	}

	dbInvoice := &models.PaymentInvoice{
		BotUserID:        botUser.ID,
		OriginalAmount:   amount,
		Amount:           invoiceAmount,
		Status:           models.InvoiceStatusPending,
		Gateway:          gatewayName,
		GatewayInvoiceID: externalInvoice.GatewayInvoiceID,
		OrderID:          orderID,
	}

	if err := s.invoiceRepo.Create(dbInvoice); err != nil {
		return nil, apperrors.New(http.StatusInternalServerError, "Failed to save invoice", err)
	}

	return &CreateInvoiceResponse{
		PayURL:           externalInvoice.PayURL,
		GatewayInvoiceID: externalInvoice.GatewayInvoiceID,
		OrderID:          orderID,
		Details:          externalInvoice.Details,
	}, nil
}

func (s *paymentService) SetInvoiceMessageID(orderID string, messageID int64) error {
	invoice, err := s.invoiceRepo.FindByOrderID(orderID)
	if err != nil {
		return &apperrors.ErrNotFound{Resource: "PaymentInvoice", IDString: orderID}
	}

	invoice.BotMessageID = &messageID
	return s.invoiceRepo.Update(invoice)
}

// processCompletedInvoice handles the business logic for a completed invoice.
// This method is designed to be called within a database transaction.
func (s *paymentService) processCompletedInvoice(tx *gorm.DB, orderID string) error {
	invoiceRepo := s.invoiceRepo.WithTx(tx)
	txnRepo := s.transactionRepo.WithTx(tx)

	invoice, err := invoiceRepo.FindByOrderID(orderID)
	if err != nil {
		return fmt.Errorf("could not find invoice with order_id %s: %w", orderID, err)
	}

	if invoice.Status == models.InvoiceStatusCompleted {
		slog.Info("invoice already completed, skipping", "order_id", orderID)
		return nil // Not an error, just already done.
	}

	if invoice.Status == models.InvoiceStatusFailed {
		return fmt.Errorf("invoice %s was already marked as failed", invoice.OrderID)
	}

	invoice.Status = models.InvoiceStatusCompleted
	if err := invoiceRepo.Update(invoice); err != nil {
		return fmt.Errorf("failed to update invoice status: %w", err)
	}

	gateway, err := s.registry.GetProvider(invoice.Gateway)
	if err != nil {
		slog.Error("could not find gateway for completed invoice", "gateway", invoice.Gateway, "order_id", orderID)
	}
	gatewayName := "N/A"
	if gateway != nil {
		gatewayName = gateway.GetDisplayName()
	}

	// --- Gateway Discount Logic ---
	allSettings, err := s.settingService.GetSettings()
	if err != nil {
		slog.Error("could not retrieve settings to check for gateway discount", "error", err)
	}

	discountKey := "GATEWAY_DISCOUNT_" + invoice.Gateway
	discountPercentageStr, hasDiscount := allSettings[discountKey]

	depositAmount := invoice.OriginalAmount
	notificationMessage := fmt.Sprintf("✅ Ваш баланс успешно пополнен на %.2f ₽.", depositAmount)
	description := fmt.Sprintf("Пополнение баланса через %s (Счет: %s)", gatewayName, invoice.GatewayInvoiceID)

	if hasDiscount && err == nil {
		if discountPercentage, parseErr := strconv.ParseFloat(discountPercentageStr, 64); parseErr == nil && discountPercentage > 0 {
			description = fmt.Sprintf("Пополнение через %s (Счет: %s). Скидка %.2f%%.", gatewayName, invoice.GatewayInvoiceID, discountPercentage)
			notificationMessage = fmt.Sprintf("✅ Ваш баланс пополнен на %.2f ₽ (с учетом скидки %.2f%%).", depositAmount, discountPercentage)
		}
	}
	// --- End Gateway Discount Logic ---

	depositTx := &models.Transaction{
		UserID:      invoice.BotUserID,
		Type:        models.Deposit,
		Amount:      depositAmount,
		Description: description,
	}

	if err := txnRepo.CreateTransaction(depositTx); err != nil {
		return fmt.Errorf("failed to create deposit transaction: %w", err)
	}

	if err := s.botUserRepo.WithTx(tx).UpdateBalance(invoice.BotUserID, depositAmount); err != nil {
		return fmt.Errorf("failed to update user balance: %w", err)
	}

	user, err := s.botUserRepo.WithTx(tx).FindByID(invoice.BotUserID)
	if err != nil {
		slog.Error("could not find user to notify about payment", "userID", invoice.BotUserID, "error", err)
	} else {
		go func() {
			// We pass nil for messageToEdit and the original message ID to messageToDelete to trigger a delete-and-send-new action in the bot.
			if err := s.webhookService.SendNotification(user.LastSeenWithBot, user.TelegramID, notificationMessage, nil, invoice.BotMessageID); err != nil {
				slog.Error("failed to send payment notification webhook", "userID", user.ID, "error", err)
			}
		}()
	}

	return nil
}

func (s *paymentService) HandleWebhook(gatewayName string, r *http.Request) error {
	gateway, err := s.registry.GetProvider(gatewayName)
	if err != nil {
		return apperrors.New(http.StatusNotFound, "Gateway not found", err)
	}

	webhookResult, err := gateway.HandleWebhook(r)
	if err != nil {
		return apperrors.New(http.StatusBadRequest, "Webhook handling failed", err)
	}

	// Handle cases where webhook is not supported or event is not relevant
	if webhookResult == nil || webhookResult.Status != string(models.InvoiceStatusCompleted) {
		return nil
	}

	txErr := s.db.Transaction(func(tx *gorm.DB) error {
		return s.processCompletedInvoice(tx, webhookResult.OrderID)
	})

	if txErr != nil {
		return apperrors.New(http.StatusInternalServerError, "Failed to process webhook transaction", txErr)
	}

	return nil
}

func (s *paymentService) PollPendingPayments() error {
	slog.Debug("starting payment polling job")
	invoices, err := s.invoiceRepo.GetPendingInvoices()
	if err != nil {
		return fmt.Errorf("failed to get pending invoices for polling: %w", err)
	}

	if len(invoices) > 0 {
		slog.Info("found pending invoices to poll", "count", len(invoices))
	}

	for _, invoice := range invoices {
		gateway, err := s.registry.GetProvider(invoice.Gateway)
		if err != nil {
			slog.Error("polling: could not find gateway for invoice", "gateway", invoice.Gateway, "order_id", invoice.OrderID)
			continue
		}

		statusResult, err := gateway.GetInvoiceStatus(invoice.GatewayInvoiceID)
		if err != nil {
			slog.Error("polling: failed to get invoice status from gateway", "gateway", invoice.Gateway, "order_id", invoice.OrderID, "error", err)
			continue
		}

		if statusResult != nil && statusResult.Status == string(models.InvoiceStatusCompleted) {
			slog.Info("polling: found completed payment", "gateway", invoice.Gateway, "order_id", invoice.OrderID)
			txErr := s.db.Transaction(func(tx *gorm.DB) error {
				return s.processCompletedInvoice(tx, invoice.OrderID)
			})
			if txErr != nil {
				slog.Error("polling: failed to process completed payment", "gateway", invoice.Gateway, "order_id", invoice.OrderID, "error", txErr)
			}
		}
	}

	slog.Debug("payment polling job finished")
	return nil
}

func (s *paymentService) NotifyUnfinishedPayments() error {
	minutes := s.config.PaymentNotificationMinutes
	if minutes <= 0 {
		slog.Debug("payment notification is disabled")
		return nil
	}

	invoices, err := s.invoiceRepo.GetPendingInvoicesOlderThan(minutes)
	if err != nil {
		return fmt.Errorf("failed to get pending invoices: %w", err)
	}

	if len(invoices) == 0 {
		return nil
	}

	slog.Info("found unfinished payments to notify", "count", len(invoices))

	for _, invoice := range invoices {
		// Use a copy of the invoice in the goroutine
		invoiceCopy := invoice

		go func() {
			message := fmt.Sprintf(
				"Вы недавно пытались пополнить баланс на %.2f ₽. Возникли ли у вас какие-либо проблемы с оплатой?",
				invoiceCopy.Amount,
			)

			// Send notification
			err := s.webhookService.SendNotification(
				invoiceCopy.BotUser.LastSeenWithBot,
				invoiceCopy.BotUser.TelegramID,
				message,
				nil, // No message to edit
				nil, // No message to delete
			)
			if err != nil {
				slog.Error("failed to send payment notification", "invoice_id", invoiceCopy.ID, "error", err)
				// Continue to next invoice even if one fails
				return
			}

			// Mark invoice as notified
			invoiceCopy.WasNotificationSent = true
			if err := s.invoiceRepo.Update(&invoiceCopy); err != nil {
				slog.Error("failed to update invoice notification status", "invoice_id", invoiceCopy.ID, "error", err)
			} else {
				slog.Info("successfully sent unfinished payment notification", "invoice_id", invoiceCopy.ID, "user_id", invoiceCopy.BotUserID)
			}
		}()
	}

	return nil
}
