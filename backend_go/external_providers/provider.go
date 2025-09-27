package external_providers

import (
	"frbktg/backend_go/models"
	"time"
)

// ProviderProduct represents a product or service from an external provider.
type ProviderProduct struct {
	ExternalID  string
	Name        string
	Price       float64
	Description string
	Type        string // e.g., "item", "subscription"
	// Metadata can store any other provider-specific information.
	Metadata map[string]interface{}
}

// ProvisioningResult holds the information about a newly provisioned subscription.
type ProvisioningResult struct {
	ProvisionedID string // The unique ID for this instance (e.g., a username, a server ID).
	Details       map[string]interface{} // IP address, password, etc.
}

// StatusResult holds the status of a provisioned subscription.
type StatusResult struct {
	IsActive  bool
	ExpiresAt time.Time
	Details   map[string]interface{}
}

// ItemPurchaseResult holds the result of purchasing a one-time item.
type ItemPurchaseResult struct {
	Details map[string]interface{} // e.g., a license key, a download link
}

// ExternalProductProvider is the base interface for all external providers.
type ExternalProductProvider interface {
	// GetName returns the unique name of the provider (e.g., "contms_proxy").
	GetName() string

	// GetProducts fetches available products from the external provider.
	GetProducts() ([]ProviderProduct, error)
}

// SubscriptionProvider is an interface for providers that manage subscriptions.
type SubscriptionProvider interface {
	ExternalProductProvider // Embeds the base interface

	ProvisionSubscription(productExternalID string, user models.BotUser, duration time.Duration) (*ProvisioningResult, error)
	DeprovisionSubscription(provisionedID string) error
	RenewSubscription(provisionedID string, duration time.Duration) error
	GetSubscriptionStatus(provisionedID string) (*StatusResult, error)
}

// ItemProvider is an interface for providers that sell one-time items.
type ItemProvider interface {
	ExternalProductProvider // Embeds the base interface

	PurchaseItem(productExternalID string, user models.BotUser) (*ItemPurchaseResult, error)
}