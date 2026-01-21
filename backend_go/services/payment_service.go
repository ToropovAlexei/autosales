package services

import (
	"encoding/json"
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
	GetInvoicesByTelegramID(telegramID int64, page models.Page) (*models.PaginatedResult[models.PaymentInvoice], error)
	GetInvoiceByID(invoiceID uint) (*models.PaymentInvoice, error)
	SetInvoiceMessageID(orderID string, messageID int64) error
	HandleWebhook(gatewayName string, r *http.Request) error
	NotifyUnfinishedPayments() error
	PollPendingPayments() error
	ConfirmExternalPayment(orderID string) error
	CancelExternalPayment(orderID string) error
	SubmitReceiptLink(orderID string, receiptURL string) error
}

type paymentService struct {
	db                  *gorm.DB
	registry            *gateways.ProviderRegistry
	invoiceRepo         repositories.PaymentInvoiceRepository
	transactionRepo     repositories.TransactionRepository
	botUserRepo         repositories.BotUserRepository
	webhookService      WebhookService
	settingService      *SettingService
	config              *config.Config
	storeBalanceService StoreBalanceService
}

func NewPaymentService(db *gorm.DB, registry *gateways.ProviderRegistry, invoiceRepo repositories.PaymentInvoiceRepository, transactionRepo repositories.TransactionRepository, botUserRepo repositories.BotUserRepository, webhookService WebhookService, settingService *SettingService, config *config.Config, storeBalanceService StoreBalanceService) PaymentService {
	return &paymentService{
		db:                  db,
		registry:            registry,
		invoiceRepo:         invoiceRepo,
		transactionRepo:     transactionRepo,
		botUserRepo:         botUserRepo,
		webhookService:      webhookService,
		settingService:      settingService,
		config:              config,
		storeBalanceService: storeBalanceService,
	}
}

func (s *paymentService) ConfirmExternalPayment(orderID string) error {
	invoice, err := s.invoiceRepo.FindByOrderID(orderID)
	if err != nil {
		return &apperrors.ErrNotFound{Resource: "PaymentInvoice", IDString: orderID}
	}

	if invoice.Gateway != "platform_card" && invoice.Gateway != "platform_sbp" {
		return apperrors.New(http.StatusBadRequest, "operation not supported for this gateway", nil)
	}

	gateway, err := s.registry.GetProvider(invoice.Gateway)
	if err != nil {
		return apperrors.New(http.StatusInternalServerError, "gateway not found", err)
	}

	ppsAdapter, ok := gateway.(*gateways.PlatformPaymentSystemAdapter)
	if !ok {
		return apperrors.New(http.StatusInternalServerError, "invalid gateway adapter type", nil)
	}

	// Use a transaction for atomicity
	err = s.db.Transaction(func(tx *gorm.DB) error {
		// Perform the external confirmation
		if err := ppsAdapter.ConfirmPayment(invoice.GatewayInvoiceID); err != nil {
			return err // Rollback transaction on external API error
		}

		// Update local invoice status to prevent further notifications
		invoice.Status = models.InvoiceStatusManuallyConfirmed
		if err := s.invoiceRepo.WithTx(tx).Update(invoice); err != nil {
			return fmt.Errorf("failed to update invoice status to manually_confirmed: %w", err)
		}
		return nil
	})

	if err != nil {
		return fmt.Errorf("failed to confirm external payment: %w", err)
	}

	return nil
}

func (s *paymentService) CancelExternalPayment(orderID string) error {
	invoice, err := s.invoiceRepo.FindByOrderID(orderID)
	if err != nil {
		return &apperrors.ErrNotFound{Resource: "PaymentInvoice", IDString: orderID}
	}

	if invoice.Gateway != "platform_card" && invoice.Gateway != "platform_sbp" {
		return apperrors.New(http.StatusBadRequest, "operation not supported for this gateway", nil)
	}

	gateway, err := s.registry.GetProvider(invoice.Gateway)
	if err != nil {
		return apperrors.New(http.StatusInternalServerError, "gateway not found", err)
	}

	ppsAdapter, ok := gateway.(*gateways.PlatformPaymentSystemAdapter)
	if !ok {
		return apperrors.New(http.StatusInternalServerError, "invalid gateway adapter type", nil)
	}

	return ppsAdapter.CancelPayment(invoice.GatewayInvoiceID)
}

func (s *paymentService) GetAvailableGateways() []gateways.PaymentGateway {
	return s.registry.GetAllProviders()
}

func (s *paymentService) GetInvoicesByTelegramID(telegramID int64, page models.Page) (*models.PaginatedResult[models.PaymentInvoice], error) {
	return s.invoiceRepo.FindInvoicesByTelegramID(telegramID, page)
}

func (s *paymentService) GetInvoiceByID(invoiceID uint) (*models.PaymentInvoice, error) {
	return s.invoiceRepo.FindByID(invoiceID)
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

	// Marshal externalInvoice.Details to JSON for storage
	var paymentDetailsJSON []byte
	if externalInvoice.Details != nil {
		paymentDetailsJSON, err = json.Marshal(externalInvoice.Details)
		if err != nil {
			return nil, apperrors.New(http.StatusInternalServerError, "Failed to marshal payment details", err)
		}
	}

	dbInvoice := &models.PaymentInvoice{
		BotUserID:        botUser.ID,
		OriginalAmount:   amount,
		Amount:           invoiceAmount,
		Status:           models.InvoiceStatusPending,
		Gateway:          gatewayName,
		GatewayInvoiceID: externalInvoice.GatewayInvoiceID,
		OrderID:          orderID,
		PayURL:           externalInvoice.PayURL, // Store PayURL
		PaymentDetails:   paymentDetailsJSON,     // Store marshaled Details
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
	botUserRepo := s.botUserRepo.WithTx(tx)

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
		// Continue without gateway-specific logic if it's not found, but log it as an error.
	}

	allSettings, err := (*s.settingService).GetSettings()
	if err != nil {
		slog.Error("could not retrieve settings for bonus/commission logic", "error", err)
		// Continue without bonus/commission, but this is a significant issue.
	}

	// --- Commission Logic ---
	// Using hardcoded values as requested
	const serviceUSDRate = 1.0
	const autosaleCommissionPercent = 1.0
	const cryptoProviderCommissionPercent = 0.05
	const paymentProviderCommissionPercent = 0.2
	const platformOnlyCommissionPercent = 1.0

	amountUSD := invoice.Amount / serviceUSDRate

	var platformCommission float64
	var gatewayCommission float64

	switch invoice.Gateway {
	case "mock_provider": // Криптопроц
		platformCommission = amountUSD * (autosaleCommissionPercent / 100.0)
		gatewayCommission = amountUSD * (cryptoProviderCommissionPercent / 100.0)
	default: // Платежки (and other providers not specifically handled, will use default payment provider comms)
		platformCommission = amountUSD * (autosaleCommissionPercent / 100.0)
		gatewayCommission = amountUSD * (paymentProviderCommissionPercent / 100.0)
	}

	// For the "Сторонняя платежка" case, if a specific gateway were identified:
	// case "some_third_party_gateway":
	//     platformCommission = amountUSD * (platformOnlyCommissionPercent / 100.0)
	//     gatewayCommission = 0.0

	storeBalanceDelta := amountUSD - platformCommission - gatewayCommission
	// --- End Commission Logic ---

	// --- Bonus Logic ---
	bonusKey := "GATEWAY_BONUS_" + invoice.Gateway
	bonusPercentageStr, hasBonus := allSettings[bonusKey]
	bonusPercentage, _ := strconv.ParseFloat(bonusPercentageStr, 64)
	finalAmount := invoice.Amount
	if hasBonus && bonusPercentage > 0 {
		finalAmount = invoice.Amount * (1 + bonusPercentage/100.0)
	}
	// --- End Bonus Logic ---

	transaction := &models.Transaction{
		UserID:             invoice.BotUserID,
		Type:               models.Deposit,
		Amount:             finalAmount,
		Description:        fmt.Sprintf("Пополнение баланса через %s", gateway.GetDisplayName()),
		PaymentGateway:     invoice.Gateway,
		GatewayCommission:  gatewayCommission,
		PlatformCommission: platformCommission,
		StoreBalanceDelta:  storeBalanceDelta,
	}

	if err := txnRepo.CreateTransaction(transaction); err != nil {
		return fmt.Errorf("failed to create deposit transaction: %w", err)
	}

	if err := botUserRepo.UpdateBalance(invoice.BotUserID, finalAmount); err != nil {
		return fmt.Errorf("failed to update user balance: %w", err)
	}

	// Update store balance
	if err := s.storeBalanceService.UpdateStoreBalance(tx, storeBalanceDelta); err != nil {
		return fmt.Errorf("failed to update store balance: %w", err)
	}

	slog.Info("successfully processed completed invoice", "order_id", orderID, "user_id", invoice.BotUserID, "amount", finalAmount)

	// Send notification to bot
	if err := s.webhookService.SendSuccessfulPaymentNotification(invoice.BotUser.RegisteredWithBot, invoice.BotUser.TelegramID, finalAmount, int(bonusPercentage), invoice.BotMessageID); err != nil {
		slog.Error("failed to send successful payment notification", "error", err, "user_id", invoice.BotUserID)
		// Do not return error, as the payment is already processed
	}

	return nil
}

func (s *paymentService) HandleWebhook(gatewayName string, r *http.Request) error {
	gateway, err := s.registry.GetProvider(gatewayName)
	if err != nil {
		return apperrors.New(http.StatusBadRequest, "Invalid payment gateway", err)
	}

	webhookHandler, ok := gateway.(gateways.WebhookHandler)
	if !ok {
		return apperrors.New(http.StatusNotImplemented, "Webhook not supported for this gateway", nil)
	}

	orderID, err := webhookHandler.HandleWebhook(r)
	if err != nil {
		return apperrors.New(http.StatusInternalServerError, "Failed to handle webhook", err)
	}

	return s.db.Transaction(func(tx *gorm.DB) error {
		return s.processCompletedInvoice(tx, orderID)
	})
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
			// Re-fetch the invoice to get the most current status to avoid race conditions
			freshInvoice, findErr := s.invoiceRepo.FindByID(invoiceCopy.ID)
			if findErr != nil {
				slog.Error("failed to re-fetch invoice for notification check", "invoice_id", invoiceCopy.ID, "error", findErr)
				return // Don't proceed if we can't get the latest state
			}

			// Only send notification if the payment is still pending
			if freshInvoice.Status != models.InvoiceStatusPending {
				slog.Info("skipping unfinished payment notification as status is no longer pending", "invoice_id", freshInvoice.ID, "status", freshInvoice.Status)
				return
			}

			message := fmt.Sprintf(
				"Вы недавно пытались пополнить баланс на %.2f ₽. Возникли ли у вас какие-либо проблемы с оплатой?",
				invoiceCopy.Amount,
			)

			var keyboard [][]InlineKeyboardButton
			if invoiceCopy.Gateway == "platform_card" || invoiceCopy.Gateway == "platform_sbp" {
				keyboard = [][]InlineKeyboardButton{
					{
						{Text: "Я все оплатил", CallbackData: fmt.Sprintf("payment_confirm:%s", invoiceCopy.OrderID)},
						{Text: "Отменить платеж", CallbackData: fmt.Sprintf("payment_cancel:%s", invoiceCopy.OrderID)},
					},
				}
			}

			// Send notification
			err := s.webhookService.SendNotification(
				invoiceCopy.BotUser.LastSeenWithBot,
				invoiceCopy.BotUser.TelegramID,
				message,
				nil, // No message to edit
				nil, // No message to delete
				keyboard,
				nil, // stateToSet
				nil, // stateData
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

func (s *paymentService) PollPendingPayments() error {
	invoices, err := s.invoiceRepo.FindPendingPollable()
	if err != nil {
		slog.Error("failed to find pollable invoices", "error", err)
		return err
	}

	slog.Info("found pending pollable invoices", "count", len(invoices))

	for _, invoice := range invoices {
		gateway, err := s.registry.GetProvider(invoice.Gateway)
		if err != nil {
			slog.Error("gateway not found for polling", "gateway", invoice.Gateway, "order_id", invoice.OrderID)
			continue
		}

		pollableGateway, ok := gateway.(gateways.Pollable)
		if !ok {
			slog.Warn("gateway is not pollable, but invoice was marked as such", "gateway", invoice.Gateway, "order_id", invoice.OrderID)
			continue
		}

		status, err := pollableGateway.GetInvoiceStatus(invoice.GatewayInvoiceID)
		if err != nil {
			slog.Error("failed to get invoice status from gateway", "gateway", invoice.Gateway, "order_id", invoice.OrderID, "error", err)
			continue
		}

		slog.Info("polled invoice status", "gateway", invoice.Gateway, "order_id", invoice.OrderID, "status", status)

		if status == gateways.InvoiceStatusCompleted {
			err := s.db.Transaction(func(tx *gorm.DB) error {
				return s.processCompletedInvoice(tx, invoice.OrderID)
			})
			if err != nil {
				slog.Error("failed to process completed invoice from polling", "gateway", invoice.Gateway, "order_id", invoice.OrderID, "error", err)
			}
		} else if status == gateways.InvoiceStatusCheckRequired {
			// Update status to awaiting_check to prevent sending notifications repeatedly
			invoice.Status = models.InvoiceStatusAwaitingCheck
			if err := s.invoiceRepo.Update(&invoice); err != nil {
				slog.Error("failed to update invoice status to awaiting_check", "gateway", invoice.Gateway, "order_id", invoice.OrderID, "error", err)
				continue
			}
			// Notify user to send a check
			if err := s.webhookService.SendCheckRequestNotification(invoice.BotUser.RegisteredWithBot, invoice.BotUser.TelegramID, invoice.OrderID); err != nil {
				slog.Error("failed to send check request notification", "gateway", invoice.Gateway, "order_id", invoice.OrderID, "error", err)
			}
		} else if status == gateways.InvoiceStatusAppeal {
			invoice.Status = models.InvoiceStatusFailed // Stop polling
			if err := s.invoiceRepo.Update(&invoice); err != nil {
				slog.Error("failed to update invoice status to failed after appeal", "gateway", invoice.Gateway, "order_id", invoice.OrderID, "error", err)
				continue
			}
			if err := s.webhookService.SendPaymentAppealNotification(invoice.BotUser.RegisteredWithBot, invoice.BotUser.TelegramID, invoice.OrderID); err != nil {
				slog.Error("failed to send payment appeal notification", "gateway", invoice.Gateway, "order_id", invoice.OrderID, "error", err)
			}
		} else if status == gateways.InvoiceStatusFailed || status == gateways.InvoiceStatusRejected {
			invoice.Status = models.InvoiceStatusFailed
			if err := s.invoiceRepo.Update(&invoice); err != nil {
				slog.Error("failed to update invoice status to failed from polling", "gateway", invoice.Gateway, "order_id", invoice.OrderID, "error", err)
			}
		}
	}

	return nil
}

func (s *paymentService) SubmitReceiptLink(orderID string, receiptURL string) error {
	invoice, err := s.invoiceRepo.FindByOrderID(orderID)
	if err != nil {
		return &apperrors.ErrNotFound{Resource: "PaymentInvoice", IDString: orderID}
	}

	if invoice.Status != models.InvoiceStatusAwaitingCheck {
		return apperrors.New(http.StatusConflict, "invoice is not awaiting a check", nil)
	}

	gateway, err := s.registry.GetProvider(invoice.Gateway)
	if err != nil {
		return apperrors.New(http.StatusInternalServerError, "gateway not found", err)
	}

	ppsAdapter, ok := gateway.(*gateways.PlatformPaymentSystemAdapter)
	if !ok {
		return apperrors.New(http.StatusInternalServerError, "invalid gateway adapter type", nil)
	}

	if err := ppsAdapter.SubmitReceipt(invoice.GatewayInvoiceID, receiptURL); err != nil {
		return apperrors.New(http.StatusInternalServerError, "failed to submit receipt to gateway", err)
	}

	// Set status back to pending to re-enter polling cycle
	invoice.Status = models.InvoiceStatusPending
	if err := s.invoiceRepo.Update(invoice); err != nil {
		return apperrors.New(http.StatusInternalServerError, "failed to update invoice status after submitting receipt", err)
	}

	slog.Info("successfully submitted receipt link", "order_id", orderID, "url", receiptURL)

	return nil
}
