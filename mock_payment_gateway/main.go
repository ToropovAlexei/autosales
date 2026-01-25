package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"html/template"
	"log"
	"net/http"
	"os"
	"sync"
	"time"

	"github.com/google/uuid"
)

var (
	mainBackendWebhookURL = getEnv("MAIN_BACKEND_WEBHOOK_URL", "http://localhost:8000/api/webhook/payment/mock-provider")
	port                  = getEnv("PORT", "8078")
	publicURL             = getEnv("MOCK_GATEWAY_PUBLIC_URL", "http://localhost:8078")
)

// Invoice represents a payment request from the main backend
type Invoice struct {
	ID        string    `json:"id"` // The mock gateway's own invoice ID (UUID)
	Amount    float64   `json:"amount"`
	UserID    uint      `json:"user_id"`
	OrderID   string    `json:"order_id"` // The unique ID from our main backend
	Status    string    `json:"status"`
	PayURL    string    `json:"pay_url"`
	CreatedAt time.Time `json:"created_at"`
}

// Store invoices in memory (for mocking purposes)
var (
	invoices = make(map[string]*Invoice)
	mu       sync.RWMutex
)

// --- Handlers ---

// createInvoiceHandler handles requests from our main backend to create a payment invoice.
func createInvoiceHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Invalid request method", http.StatusMethodNotAllowed)
		return
	}

	var req struct {
		Amount  float64 `json:"amount"`
		UserID  uint    `json:"user_id"`
		OrderID string  `json:"order_id"`
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	log.Printf("Using public URL: %s", publicURL)

	invoiceID := uuid.New().String()
	invoice := &Invoice{
		ID:        invoiceID,
		Amount:    req.Amount,
		UserID:    req.UserID,
		OrderID:   req.OrderID,
		Status:    "pending",
		PayURL:    fmt.Sprintf("%s/pay/%s", publicURL, invoiceID),
		CreatedAt: time.Now(),
	}

	mu.Lock()
	invoices[invoiceID] = invoice
	mu.Unlock()

	log.Printf("Created invoice %s for order %s (Amount: %.2f)", invoiceID, req.OrderID, req.Amount)

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{
		"invoice_id": invoiceID,
		"pay_url":    invoice.PayURL,
	})
}

// servePaymentPageHandler serves the HTML page where the user "pays".
func servePaymentPageHandler(w http.ResponseWriter, r *http.Request) {
	invoiceID := r.URL.Path[len("/pay/"):]
	mu.RLock()
	invoice, ok := invoices[invoiceID]
	mu.RUnlock()

	if !ok {
		http.Error(w, "Invoice not found", http.StatusNotFound)
		return
	}

	tmpl, err := template.ParseFiles("payment.html")
	if err != nil {
		http.Error(w, "Could not load payment page template", http.StatusInternalServerError)
		log.Printf("Error parsing template: %v", err)
		return
	}
	tmpl.Execute(w, invoice)
}

// simulatePaymentHandler is called by the payment page to simulate a successful payment.
func simulatePaymentHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Invalid request method", http.StatusMethodNotAllowed)
		return
	}

	invoiceID := r.URL.Path[len("/simulate_payment/"):]
	mu.Lock()
	invoice, ok := invoices[invoiceID]
	if ok {
		invoice.Status = "completed"
	}
	mu.Unlock()

	if !ok {
		http.Error(w, "Invoice not found", http.StatusNotFound)
		return
	}

	log.Printf("Simulating successful payment for invoice %s. Sending webhook...", invoiceID)

	// Send webhook to the main backend
	go func() {
		webhookPayload := map[string]interface{}{
			"event":      "payment.completed",
			"order_id":   invoice.OrderID,
			"invoice_id": invoiceID,
			"amount":     invoice.Amount,
			"status":     "completed",
		}
		payloadBytes, _ := json.Marshal(webhookPayload)

		_, err := http.Post(mainBackendWebhookURL, "application/json", bytes.NewBuffer(payloadBytes))
		if err != nil {
			log.Printf("ERROR: Failed to send webhook for order %s: %v", invoice.OrderID, err)
		} else {
			log.Printf("Successfully sent webhook for order %s", invoice.OrderID)
		}
	}()

	fmt.Fprintf(w, "<h1>Payment successful!</h1><p>You can now close this window.</p>")
}

// getInvoiceStatusHandler returns the current status of an invoice.
func getInvoiceStatusHandler(w http.ResponseWriter, r *http.Request) {
	invoiceID := r.URL.Path[len("/status/"):]
	mu.RLock()
	invoice, ok := invoices[invoiceID]
	mu.RUnlock()

	if !ok {
		http.Error(w, "Invoice not found", http.StatusNotFound)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{
		"status": invoice.Status,
	})
}

func main() {
	http.HandleFunc("/create_invoice", createInvoiceHandler)
	http.HandleFunc("/pay/", servePaymentPageHandler)
	http.HandleFunc("/simulate_payment/", simulatePaymentHandler)
	http.HandleFunc("/status/", getInvoiceStatusHandler)

	log.Printf("Mock Payment Gateway starting on port %s", port)
	if err := http.ListenAndServe(":"+port, nil); err != nil {
		log.Fatalf("Failed to start server: %v", err)
	}
}

func getEnv(key, fallback string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	return fallback
}
