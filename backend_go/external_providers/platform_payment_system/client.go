package platform_payment_system

import (
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"net/url"
	"strings"
	"sync"
	"time"

	"github.com/pquerna/otp/totp"
)

// AuthStep1Response is the response from the first auth step
type AuthStep1Response struct {
	Data struct {
		Temp string `json:"temp"`
	} `json:"data"`
}

// AuthStep2Response is the response from the second auth step
type AuthStep2Response struct {
	Data struct {
		Token string `json:"token"`
	} `json:"data"`
}

// Client handles communication with the new payment provider API
type Client struct {
	httpClient *http.Client
	baseURL    string
	login      string
	password   string
	twoFAKey   string
	token      string
	mu         sync.RWMutex
}

// NewClient creates a new client for the payment provider
func NewClient(baseURL, login, password, twoFAKey string) *Client {
	return &Client{
		httpClient: &http.Client{Timeout: 15 * time.Second},
		baseURL:    baseURL,
		login:      login,
		password:   password,
		twoFAKey:   twoFAKey,
	}
}

func (c *Client) authStep1() (string, error) {
	data := url.Values{}
	data.Set("version", "1")
	data.Set("login", c.login)
	data.Set("password", c.password)

	fullURL, _ := url.JoinPath(c.baseURL, "/api/method/client/auth/step1")

	req, err := http.NewRequest("POST", fullURL, strings.NewReader(data.Encode()))
	if err != nil {
		return "", fmt.Errorf("failed to create auth step1 request: %w", err)
	}
	req.Header.Add("Content-Type", "application/x-www-form-urlencoded")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return "", fmt.Errorf("failed to perform auth step1 request: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", fmt.Errorf("failed to read auth step1 response body: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("auth step1 request returned non-OK status: %d. Body: %s", resp.StatusCode, string(body))
	}

	var genericResp GenericResponse
	if err := json.Unmarshal(body, &genericResp); err != nil {
		return "", fmt.Errorf("failed to unmarshal auth step1 generic response: %w. Body: %s", err, string(body))
	}
	if genericResp.Response != "success" {
		return "", fmt.Errorf("auth step1 was not successful: %s", genericResp.Message)
	}

	var authResp AuthStep1Response
	if err := json.Unmarshal(body, &authResp); err != nil {
		return "", fmt.Errorf("failed to unmarshal auth step1 response: %w", err)
	}

	if authResp.Data.Temp == "" {
		return "", fmt.Errorf("temp token not found in auth step1 response")
	}

	return authResp.Data.Temp, nil
}

func generateTOTP(secret string) (string, error) {
	return totp.GenerateCode(secret, time.Now())
}

func (c *Client) authStep2(tempToken string) (string, error) {
	otpCode, err := generateTOTP(c.twoFAKey)
	if err != nil {
		return "", fmt.Errorf("failed to generate TOTP code: %w", err)
	}

	data := url.Values{}
	data.Set("version", "1")
	data.Set("temp", tempToken)
	data.Set("key", otpCode)

	fullURL, _ := url.JoinPath(c.baseURL, "/api/method/client/auth/step2")

	req, err := http.NewRequest("POST", fullURL, strings.NewReader(data.Encode()))
	if err != nil {
		return "", fmt.Errorf("failed to create auth step2 request: %w", err)
	}
	req.Header.Add("Content-Type", "application/x-www-form-urlencoded")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return "", fmt.Errorf("failed to perform auth step2 request: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", fmt.Errorf("failed to read auth step2 response body: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("auth step2 request returned non-OK status: %d. Body: %s", resp.StatusCode, string(body))
	}

	var genericResp GenericResponse
	if err := json.Unmarshal(body, &genericResp); err != nil {
		return "", fmt.Errorf("failed to unmarshal auth step2 generic response: %w. Body: %s", err, string(body))
	}
	if genericResp.Response != "success" {
		return "", fmt.Errorf("auth step2 was not successful: %s", genericResp.Message)
	}

	var authResp AuthStep2Response
	if err := json.Unmarshal(body, &authResp); err != nil {
		return "", fmt.Errorf("failed to unmarshal auth step2 response: %w", err)
	}

	if authResp.Data.Token == "" {
		return "", fmt.Errorf("permanent token not found in auth step2 response")
	}

	return authResp.Data.Token, nil
}

func (c *Client) authenticate() error {
	tempToken, err := c.authStep1()
	if err != nil {
		return err
	}

	permToken, err := c.authStep2(tempToken)
	if err != nil {
		return err
	}

	c.mu.Lock()
	c.token = permToken
	c.mu.Unlock()

	slog.Info("Successfully authenticated with payment provider.")
	return nil
}

func (c *Client) getToken() (string, error) {
	c.mu.RLock()
	token := c.token
	c.mu.RUnlock()

	if token == "" {
		if err := c.authenticate(); err != nil {
			return "", err
		}
		c.mu.RLock()
		token = c.token
		c.mu.RUnlock()
	}

	return token, nil
}

// OrderInitializedResponse is the response for the order_initialized method
type OrderInitializedResponse struct {
	Data struct {
		DataRequisite struct {
			ObjectToken string `json:"object_token"`
			Value       string `json:"value"`
			DataBank    struct {
				Name string `json:"name"`
			} `json:"data_bank"`
			DataPeople struct {
				Surname    string `json:"surname"`
				Name       string `json:"name"`
				Patronymic string `json:"patronymic"`
			} `json:"data_people"`
			DataMathematics struct {
				Currency       string  `json:"currency"`
				Country        string  `json:"country"`
				AmountPay      float64 `json:"amount_pay"`
				AmountTransfer float64 `json:"amount_transfer"`
			} `json:"data_mathematics"`
		} `json:"data_requisite"`
	} `json:"data"`
}

// OrderStatusResponse is the response for the order_get_status method
type OrderStatusResponse struct {
	Data struct {
		Status struct {
			Token            string  `json:"token"`
			Status           string  `json:"status"`
			AppealFakeStatus *string `json:"appeal_fake_status"`
			AppealURLFile    string  `json:"appeal_url_file"`
			RequisiteData    struct {
				Country  string `json:"country"`
				BankImg  string `json:"bank_img"`
				BankName string `json:"bank_name"`
				Currency string `json:"currency"`
				Emoji    string `json:"emoji"`
				Value    string `json:"value"`
				Type     string `json:"type"`
			} `json:"requisite_data"`
			TokenLink                *string `json:"token_link"`
			AmountOrderRequested     string  `json:"amount_order_requested"`
			AmountOrderCharged       string  `json:"amount_order_charged"`
			DatetimeCreatedCosmetics string  `json:"datetime_created_cosmetics"`
			DatetimeCreatedDatetime  string  `json:"datetime_created_datetime"`
			TimerCosmetics           string  `json:"timer_cosmetics"`
			TimerDatetime            string  `json:"timer_datetime"`
		} `json:"status"`
	} `json:"data"`
}

// GenericResponse is a generic response for simple success messages
type GenericResponse struct {
	Response string `json:"response"`
	Message  string `json:"message"`
}

func (c *Client) doRequest(endpoint string, data url.Values, responseContainer interface{}) error {
	token, err := c.getToken()
	if err != nil {
		return err
	}
	data.Set("user_token", token)

	base, err := url.Parse(c.baseURL)
	if err != nil {
		return fmt.Errorf("invalid base URL for payment provider: %w", err)
	}

	endpointURL, err := url.Parse(endpoint)
	if err != nil {
		return fmt.Errorf("invalid endpoint for payment provider: %w", err)
	}

	fullURL := base.ResolveReference(endpointURL).String()

	slog.Info("sending request to payment provider", "url", fullURL, "data", data)

	req, err := http.NewRequest("POST", fullURL, strings.NewReader(data.Encode()))
	if err != nil {
		return fmt.Errorf("failed to create request for %s: %w", fullURL, err)
	}
	req.Header.Add("Content-Type", "application/x-www-form-urlencoded")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		slog.Error("payment provider request failed", "url", fullURL, "error", err)
		return fmt.Errorf("failed to perform request for %s: %w", fullURL, err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("failed to read response body for %s: %w", fullURL, err)
	}

	if resp.StatusCode != http.StatusOK {
		slog.Error("payment provider request returned non-OK status", "url", fullURL, "status", resp.StatusCode, "body", string(body))
		return fmt.Errorf("request for %s returned non-OK status: %d. Body: %s", fullURL, resp.StatusCode, string(body))
	}

	slog.Info("payment provider request successful", "url", fullURL, "status", resp.StatusCode)

	if responseContainer != nil {
		if err := json.Unmarshal(body, responseContainer); err != nil {
			return fmt.Errorf("failed to unmarshal response for %s: %w. Body: %s", fullURL, err, string(body))
		}
	}

	// Check for `response: "success"`
	var genericResp GenericResponse
	if err := json.Unmarshal(body, &genericResp); err == nil {
		if genericResp.Response != "success" {
			slog.Error("payment provider API call was not successful", "url", fullURL, "message", genericResp.Message)
			return fmt.Errorf("API call to %s was not successful: %s", fullURL, genericResp.Message)
		}
	}

	return nil
}

// OrderInitialized requests a new requisite
func (c *Client) OrderInitialized(amount int, idPayMethod int) (*OrderInitializedResponse, error) {
	data := url.Values{}
	data.Set("version", "3")
	data.Set("amount", fmt.Sprintf("%d", amount))
	data.Set("id_pay_method", fmt.Sprintf("%d", idPayMethod))

	var resp OrderInitializedResponse
	err := c.doRequest("/api/method/merch/payin/order_initialized/standart", data, &resp)
	if err != nil {
		return nil, err
	}

	return &resp, nil
}

// MerchProcess moves an order to processing status
func (c *Client) MerchProcess(objectToken string) error {
	data := url.Values{}
	data.Set("version", "1")
	data.Set("object_token", objectToken)

	return c.doRequest("/api/method/merch/payin/order_process", data, nil)
}

// OrderGetStatus retrieves the status of a specific order
func (c *Client) OrderGetStatus(objectToken string) (*OrderStatusResponse, error) {
	data := url.Values{}
	data.Set("version", "1")
	data.Set("object_token", objectToken)

	var resp OrderStatusResponse
	err := c.doRequest("/api/method/merch/payin/order_get_status", data, &resp)
	if err != nil {
		return nil, err
	}

	return &resp, nil
}

// OrderCancel cancels an order
func (c *Client) OrderCancel(objectToken string) error {
	data := url.Values{}
	data.Set("version", "1")
	data.Set("object_token", objectToken)

	return c.doRequest("/api/method/merch/payin/order_cancel", data, nil)
}

// OrderCheckDown submits a check/receipt for an order
func (c *Client) OrderCheckDown(objectToken string, urlFile string) error {
	data := url.Values{}
	data.Set("version", "2")
	data.Set("object_token", objectToken)
	data.Set("url_file", urlFile)

	return c.doRequest("/api/method/merch/payin/order_check_down", data, nil)
}
