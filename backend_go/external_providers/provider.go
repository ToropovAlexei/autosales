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
	// Metadata can store any other provider-specific information.
	Metadata map[string]interface{}
}

// ProvisioningResult holds the information about a newly provisioned product.
type ProvisioningResult struct {
	ProvisionedID string // The unique ID for this instance (e.g., a username, a server ID).
	Details       map[string]interface{} // IP address, password, etc.
}

// StatusResult holds the status of a provisioned product.
type StatusResult struct {
	IsActive  bool
	ExpiresAt time.Time
	Details   map[string]interface{}
}

// ExternalProductProvider defines the contract for interacting with an external service that provides products.
type ExternalProductProvider interface {
	// GetName returns the unique name of the provider (e.g., "contms_proxy").
	GetName() string

	// GetProducts fetches available products from the external provider.
	GetProducts() ([]ProviderProduct, error)

	// ProvisionProduct creates a new instance of a product for a user.
	ProvisionProduct(productExternalID string, user models.BotUser, duration time.Duration) (*ProvisioningResult, error)

	// DeprovisionProduct removes/deactivates a provisioned product.
	DeprovisionProduct(provisionedID string) error

	// RenewProduct extends the lifecycle of a provisioned product.
	RenewProduct(provisionedID string, duration time.Duration) error

	// GetProductStatus checks the status of a provisioned product.
	GetProductStatus(provisionedID string) (*StatusResult, error)
}
