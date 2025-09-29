package contms

import (
	"bytes"
	"encoding/json"
	"fmt"
	"frbktg/backend_go/external_providers"
	"frbktg/backend_go/models"
	"io"
	"net/http"
	"time"
)

const ProviderName = "contms_proxy"

// API-specific request and response structs
type Request struct {
	Action string      `json:"action"`
	Filter interface{} `json:"filter,omitempty"`
	Proxy  interface{} `json:"proxy,omitempty"`
	User   string      `json:"user,omitempty"`
	Expires int64      `json:"expires,omitempty"`
}

type AvailableProxy struct {
	Name string `json:"name"`
	Type string `json:"type"`
	Host string `json:"host"`
}

type AvailableResponse struct {
	Action string           `json:"action"`
	Proxy  []AvailableProxy `json:"proxy"`
	Status string           `json:"status"`
}

type UpResponse struct {
	Action string `json:"action"`
	User   struct {
		Proxy   string `json:"proxy"`
		Expires int64  `json:"expires"`
		Pass    string `json:"pass"`
		Name    string `json:"name"`
	} `json:"user"`
	Status string `json:"status"`
}

type DownResponse struct {
	Action string `json:"action"`
	User   string `json:"user"`
	Status string `json:"status"`
}

type StatusResponse struct {
	Action string `json:"action"`
	User   string `json:"user"`
	Proxy  struct {
		Expires int64 `json:"expires"`
	} `json:"proxy"`
	Status string `json:"status"`
}

type RenewResponse struct {
	Action string `json:"action"`
	User   string `json:"user"`
	Status string `json:"status"`
}

// ContMSProxyAdapter implements the SubscriptionProvider interface.
type ContMSProxyAdapter struct {
	client  *http.Client
	baseURL string
}

// NewContMSProxyAdapter creates a new adapter for the contms.ru API.
func NewContMSProxyAdapter(baseURL string) *ContMSProxyAdapter {
	return &ContMSProxyAdapter{
		client:  &http.Client{Timeout: 10 * time.Second},
		baseURL: baseURL,
	}
}

func (a *ContMSProxyAdapter) GetName() string {
	return ProviderName
}

func (a *ContMSProxyAdapter) doRequest(payload interface{}, responseContainer interface{}) error {
	body, err := json.Marshal(payload)
	if err != nil {
		return fmt.Errorf("failed to marshal request payload: %w", err)
	}

	req, err := http.NewRequest("POST", a.baseURL, bytes.NewBuffer(body))
	if err != nil {
		return fmt.Errorf("failed to create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := a.client.Do(req)
	if err != nil {
		return fmt.Errorf("failed to perform request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("received non-OK HTTP status: %s", resp.Status)
	}

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("failed to read response body: %w", err)
	}

	if err := json.Unmarshal(respBody, responseContainer); err != nil {
		return fmt.Errorf("failed to unmarshal response: %w", err)
	}

	return nil
}

func (a *ContMSProxyAdapter) GetProducts() ([]external_providers.ProviderProduct, error) {
	payload := Request{Action: "available"}
	var resp AvailableResponse

	if err := a.doRequest(payload, &resp); err != nil {
		return nil, err
	}

	if resp.Status != "ok" {
		return nil, fmt.Errorf("API status was not 'ok': %s", resp.Status)
	}

	var products []external_providers.ProviderProduct
	for _, p := range resp.Proxy {
		products = append(products, external_providers.ProviderProduct{
			ExternalID:  p.Name,
			Name:        fmt.Sprintf("Proxy %s (%s)", p.Name, p.Type),
			Price:       100, // Price is not in the API, so we assume a fixed price.
			Description: fmt.Sprintf("A %s proxy server at %s", p.Type, p.Host),
			Type:        "subscription",
			Category:    []string{"PROXY", p.Type},
		})
	}

	return products, nil
}

func (a *ContMSProxyAdapter) ProvisionSubscription(productExternalID string, user models.BotUser, duration time.Duration) (*external_providers.ProvisioningResult, error) {
	payload := Request{
		Action: "up",
		Proxy:  map[string]interface{}{"name": productExternalID, "expires": duration.Milliseconds()},
	}
	var resp UpResponse

	if err := a.doRequest(payload, &resp); err != nil {
		return nil, err
	}

	if resp.Status != "ok" {
		return nil, fmt.Errorf("API status was not 'ok': %s", resp.Status)
	}

	return &external_providers.ProvisioningResult{
		ProvisionedID: resp.User.Name,
		Details: map[string]interface{}{
			"username": resp.User.Name,
			"password": resp.User.Pass,
			"expires":  time.Unix(0, resp.User.Expires*int64(time.Millisecond)),
		},
	}, nil
}

func (a *ContMSProxyAdapter) DeprovisionSubscription(provisionedID string) error {
	payload := Request{Action: "down", User: provisionedID}
	var resp DownResponse

	if err := a.doRequest(payload, &resp); err != nil {
		return err
	}

	if resp.Status != "ok" {
		return fmt.Errorf("API status was not 'ok': %s", resp.Status)
	}

	return nil
}

func (a *ContMSProxyAdapter) RenewSubscription(provisionedID string, duration time.Duration) error {
	payload := Request{
		Action:  "renew",
		User:    provisionedID,
		Expires: duration.Milliseconds(),
	}
	var resp RenewResponse

	if err := a.doRequest(payload, &resp); err != nil {
		return err
	}

	if resp.Status != "ok" {
		return fmt.Errorf("API status was not 'ok': %s", resp.Status)
	}

	return nil
}

func (a *ContMSProxyAdapter) GetSubscriptionStatus(provisionedID string) (*external_providers.StatusResult, error) {
	payload := Request{Action: "status", User: provisionedID}
	var resp StatusResponse

	if err := a.doRequest(payload, &resp); err != nil {
		return nil, err
	}

	if resp.Status != "ok" {
		return nil, fmt.Errorf("API status was not 'ok': %s", resp.Status)
	}

	return &external_providers.StatusResult{
		IsActive:  true, // The API doesn't explicitly state this, but if it returns OK, we assume it's active.
		ExpiresAt: time.Unix(0, resp.Proxy.Expires*int64(time.Millisecond)),
	}, nil
}
