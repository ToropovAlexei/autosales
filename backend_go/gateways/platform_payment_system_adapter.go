package gateways

import (
	"frbktg/backend_go/external_providers/platform_payment_system"
	"frbktg/backend_go/models"
	"net/http"
)

type PlatformPaymentSystemAdapter struct {
	client      *platform_payment_system.Client
	name        string
	displayName string
	idPayMethod int
}

// NewPlatformPaymentSystemAdapter creates a new adapter for the platform payment system.
func NewPlatformPaymentSystemAdapter(client *platform_payment_system.Client, name, displayName string, idPayMethod int) PaymentGateway {
	return &PlatformPaymentSystemAdapter{
		client:      client,
		name:        name,
		displayName: displayName,
		idPayMethod: idPayMethod,
	}
}

func (a *PlatformPaymentSystemAdapter) GetName() string {
	return a.name
}

func (a *PlatformPaymentSystemAdapter) GetDisplayName() string {
	return a.displayName
}

func (a *PlatformPaymentSystemAdapter) CreateInvoice(req *InvoiceCreationRequest) (*Invoice, error) {
	resp, err := a.client.OrderInitialized(int(req.Amount), a.idPayMethod)
	if err != nil {
		return nil, err
	}

	return &Invoice{
		GatewayInvoiceID: resp.Data.DataRequisite.ObjectToken,
		PayURL:           nil, // No payment URL for this provider
		Details:          resp.Data.DataRequisite,
	}, nil
}

func (a *PlatformPaymentSystemAdapter) HandleWebhook(r *http.Request) (*WebhookResult, error) {
	// This provider does not use webhooks. Status polling will be handled by a worker.
	return nil, nil
}

func (a *PlatformPaymentSystemAdapter) GetInvoiceStatus(gatewayInvoiceID string) (*StatusResult, error) {
	resp, err := a.client.OrderGetStatus(gatewayInvoiceID)
	if err != nil {
		return nil, err
	}

	var internalStatus string

	switch resp.Data.Status.Status {
	case "trader_success", "merch_success", "system_timer_end_merch_process_success", "system_timer_end_merch_check_down_success", "admin_appeal_success":
		internalStatus = string(models.InvoiceStatusCompleted)
	case "system_timer_end_merch_initialized_cancel", "order_cancel", "merch_cancel", "system_timer_end_trader_check_query_cancel", "admin_appeal_cancel": // Assuming these are failure/cancel statuses
		internalStatus = string(models.InvoiceStatusFailed)
	default:
		internalStatus = string(models.InvoiceStatusPending)
	}

	return &StatusResult{
		Status:           internalStatus,
		GatewayInvoiceID: gatewayInvoiceID,
	}, nil
}

func (a *PlatformPaymentSystemAdapter) ConfirmPayment(gatewayInvoiceID string) error {
	return a.client.MerchProcess(gatewayInvoiceID)
}

func (a *PlatformPaymentSystemAdapter) CancelPayment(gatewayInvoiceID string) error {
	return a.client.OrderCancel(gatewayInvoiceID)
}