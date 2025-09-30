package mock

import (
	"bytes"
	"encoding/json"
	"fmt"
	"frbktg/backend_go/gateways"
	"io"
	"net/http"
	"time"
)

const (
	ProviderName    = "mock_provider"
	ProviderDisplay = "🔥🔥Тестовый платеж Криптоплатежи(скидка - 5%)🔥🔥"
)

// MockGatewayAdapter implements the gateways.PaymentGateway interface.
type MockGatewayAdapter struct {
	client  *http.Client
	baseURL string // e.g., "http://localhost:8078"
}

// NewMockGatewayAdapter creates a new adapter for the mock payment gateway.
func NewMockGatewayAdapter(baseURL string) *MockGatewayAdapter {
	return &MockGatewayAdapter{
		client:  &http.Client{Timeout: 10 * time.Second},
		baseURL: baseURL,
	}
}

func (a *MockGatewayAdapter) GetName() string {
	return ProviderName
}

func (a *MockGatewayAdapter) GetDisplayName() string {
	return ProviderDisplay
}

func (a *MockGatewayAdapter) CreateInvoice(req *gateways.InvoiceCreationRequest) (*gateways.Invoice, error) {
	apiURL := a.baseURL + "/create_invoice"

	payload, err := json.Marshal(req)
	if err != nil {
		return nil, fmt.Errorf("mock adapter: failed to marshal request: %w", err)
	}

	resp, err := a.client.Post(apiURL, "application/json", bytes.NewBuffer(payload))
	if err != nil {
		return nil, fmt.Errorf("mock adapter: failed to send request to mock server: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("mock adapter: mock server returned status %d", resp.StatusCode)
	}

	var respData struct {
		InvoiceID string `json:"invoice_id"`
		PayURL    string `json:"pay_url"`
	}

	if err := json.NewDecoder(resp.Body).Decode(&respData); err != nil {
		return nil, fmt.Errorf("mock adapter: failed to decode response from mock server: %w", err)
	}

	return &gateways.Invoice{
		GatewayInvoiceID: respData.InvoiceID,
		PayURL:           respData.PayURL,
	}, nil
}

func (a *MockGatewayAdapter) HandleWebhook(r *http.Request) (*gateways.WebhookResult, error) {
	body, err := io.ReadAll(r.Body)
	if err != nil {
		return nil, fmt.Errorf("mock adapter: failed to read webhook body: %w", err)
	}

	var webhookPayload struct {
		Event            string  `json:"event"`
		OrderID          string  `json:"order_id"`
		InvoiceID        string  `json:"invoice_id"`
		Amount           float64 `json:"amount"`
		Status           string  `json:"status"`
	}

	if err := json.Unmarshal(body, &webhookPayload); err != nil {
		return nil, fmt.Errorf("mock adapter: failed to unmarshal webhook: %w", err)
	}

	// In a real gateway, we would verify a signature here.

	if webhookPayload.Event != "payment.completed" {
		return nil, fmt.Errorf("mock adapter: received unexpected event type '%s'", webhookPayload.Event)
	}

	return &gateways.WebhookResult{
		GatewayInvoiceID: webhookPayload.InvoiceID,
		OrderID:          webhookPayload.OrderID,
		Status:           webhookPayload.Status,
		Amount:           webhookPayload.Amount,
	}, nil
}
