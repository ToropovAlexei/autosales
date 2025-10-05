package services

import (
	"fmt"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/gateways"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"log/slog"
	"net/http"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

// CreateInvoiceResponse is a custom response that includes both external gateway data and our internal OrderID
type CreateInvoiceResponse struct {
	PayURL           string `json:"pay_url"`
	GatewayInvoiceID string `json:"gateway_invoice_id"`
	OrderID          string `json:"order_id"` // Our internal ID
}

type PaymentService interface {
	GetAvailableGateways() []gateways.PaymentGateway
	CreateInvoice(userID uint, gatewayName string, amount float64) (*CreateInvoiceResponse, error)
	SetInvoiceMessageID(orderID string, messageID int64) error
	HandleWebhook(gatewayName string, r *http.Request) error
}

type paymentService struct {
	db              *gorm.DB
	registry        *gateways.ProviderRegistry
	invoiceRepo     repositories.PaymentInvoiceRepository
	transactionRepo repositories.TransactionRepository
	botUserRepo     repositories.BotUserRepository
	webhookService  WebhookService
}

func NewPaymentService(db *gorm.DB, registry *gateways.ProviderRegistry, invoiceRepo repositories.PaymentInvoiceRepository, transactionRepo repositories.TransactionRepository, botUserRepo repositories.BotUserRepository, webhookService WebhookService) PaymentService {
	return &paymentService{
		db:              db,
		registry:        registry,
		invoiceRepo:     invoiceRepo,
		transactionRepo: transactionRepo,
		botUserRepo:     botUserRepo,
		webhookService:  webhookService,
	}
}

func (s *paymentService) GetAvailableGateways() []gateways.PaymentGateway {
	return s.registry.GetAllProviders()
}

func (s *paymentService) CreateInvoice(botUserID uint, gatewayName string, amount float64) (*CreateInvoiceResponse, error) {
	gateway, err := s.registry.GetProvider(gatewayName)
	if err != nil {
		return nil, apperrors.New(http.StatusBadRequest, "Invalid payment gateway", err)
	}

	orderID := uuid.New().String()

	invoiceReq := &gateways.InvoiceCreationRequest{
		Amount:  amount,
		UserID:  botUserID,
		OrderID: orderID,
	}

	externalInvoice, err := gateway.CreateInvoice(invoiceReq)
	if err != nil {
		return nil, apperrors.New(http.StatusInternalServerError, "Failed to create external invoice", err)
	}

	dbInvoice := &models.PaymentInvoice{
		BotUserID:        botUserID,
		Amount:           amount,
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

func (s *paymentService) HandleWebhook(gatewayName string, r *http.Request) error {
	gateway, err := s.registry.GetProvider(gatewayName)
	if err != nil {
		return apperrors.New(http.StatusNotFound, "Gateway not found", err)
	}

	webhookResult, err := gateway.HandleWebhook(r)
	if err != nil {
		return apperrors.New(http.StatusBadRequest, "Webhook handling failed", err)
	}

	if webhookResult.Status != "completed" {
		return nil
	}

	var userToNotify *models.BotUser
	var processedAmount float64
	var messageToEdit *int64

	txErr := s.db.Transaction(func(tx *gorm.DB) error {
		invoiceRepo := s.invoiceRepo.WithTx(tx)
		txnRepo := s.transactionRepo.WithTx(tx)

		invoice, err := invoiceRepo.FindByOrderID(webhookResult.OrderID)
		if err != nil {
			return fmt.Errorf("could not find invoice with order_id %s: %w", webhookResult.OrderID, err)
		}

		if invoice.Status == models.InvoiceStatusCompleted {
			return nil
		}

		if invoice.Status == models.InvoiceStatusFailed {
			return fmt.Errorf("invoice %s was already marked as failed", invoice.OrderID)
		}

		invoice.Status = models.InvoiceStatusCompleted
		if err := invoiceRepo.Update(invoice); err != nil {
			return fmt.Errorf("failed to update invoice status: %w", err)
		}

		depositTx := &models.Transaction{
			UserID:      invoice.BotUserID,
			Type:        models.Deposit,
			Amount:      invoice.Amount,
			Description: fmt.Sprintf("Пополнение баланса через %s (Счет: %s)", gateway.GetDisplayName(), invoice.GatewayInvoiceID),
		}

		if err := txnRepo.CreateTransaction(depositTx); err != nil {
			return fmt.Errorf("failed to create deposit transaction: %w", err)
		}

		user, err := s.botUserRepo.FindByID(invoice.BotUserID)
		if err != nil {
			slog.Error("could not find user to notify about payment", "userID", invoice.BotUserID, "error", err)
		} else {
			userToNotify = user
			processedAmount = invoice.Amount
			messageToEdit = invoice.BotMessageID
		}

		return nil
	})

	if txErr != nil {
		return apperrors.New(http.StatusInternalServerError, "Failed to process webhook transaction", txErr)
	}

	if userToNotify != nil {
		message := fmt.Sprintf("✅ Ваш баланс успешно пополнен на %.2f ₽.", processedAmount)
		go func() {
			if err := s.webhookService.SendNotification(userToNotify.LastSeenWithBot, userToNotify.TelegramID, message, messageToEdit); err != nil {
				slog.Error("failed to send payment notification webhook", "userID", userToNotify.ID, "error", err)
			}
		}()
	}

	return nil
}
