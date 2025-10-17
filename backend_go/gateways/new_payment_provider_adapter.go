package gateways

import (
	"frbktg/backend_go/external_providers/new_payment_provider"
	"frbktg/backend_go/models"
	"net/http"
)

const (
	NewPaymentProviderName        = "new_payment_provider"
	NewPaymentProviderDisplayName = "Новый платежный провайдер"
)

type newPaymentProviderAdapter struct {
	client *new_payment_provider.Client
}

func NewNewPaymentProviderAdapter(client *new_payment_provider.Client) PaymentGateway {
	return &newPaymentProviderAdapter{client: client}
}

func (a *newPaymentProviderAdapter) GetName() string {
	return NewPaymentProviderName
}

func (a *newPaymentProviderAdapter) GetDisplayName() string {
	return NewPaymentProviderDisplayName
}

func (a *newPaymentProviderAdapter) CreateInvoice(req *InvoiceCreationRequest) (*Invoice, error) {
	// The new provider requires id_pay_method, which is not in the standard request.
	// For now, we'll hardcode it to 1 (Карта). This might need to be passed from the frontend later.
	idPayMethod := 1

	resp, err := a.client.OrderInitialized(int(req.Amount), idPayMethod)
	if err != nil {
		return nil, err
	}

	return &Invoice{
		GatewayInvoiceID: resp.Data.DataRequisite.ObjectToken,
		PayURL:           "", // No payment URL for this provider
		Details:          resp.Data.DataRequisite,
	}, nil
}

func (a *newPaymentProviderAdapter) HandleWebhook(r *http.Request) (*WebhookResult, error) {
	// This provider does not use webhooks. Status polling will be handled by a worker.
	return nil, nil
}

func (a *newPaymentProviderAdapter) GetInvoiceStatus(gatewayInvoiceID string) (*StatusResult, error) {
	resp, err := a.client.OrderGetStatus(gatewayInvoiceID)
	if err != nil {
		return nil, err
	}

	var internalStatus string

	// TODO: Refine status mapping once the provider's status values are known.
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
