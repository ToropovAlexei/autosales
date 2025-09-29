package gateways

import "fmt"

// ProviderRegistry holds all registered payment gateway providers.
type ProviderRegistry struct {
	providers map[string]PaymentGateway
}

// NewProviderRegistry creates a new, empty registry.
func NewProviderRegistry() *ProviderRegistry {
	return &ProviderRegistry{
		providers: make(map[string]PaymentGateway),
	}
}

// RegisterProvider adds a provider to the registry.
func (r *ProviderRegistry) RegisterProvider(provider PaymentGateway) {
	r.providers[provider.GetName()] = provider
}

// GetProvider retrieves a provider by its name.
func (r *ProviderRegistry) GetProvider(name string) (PaymentGateway, error) {
	provider, ok := r.providers[name]
	if !ok {
		return nil, fmt.Errorf("provider not found: %s", name)
	}
	return provider, nil
}

// GetAllProviders returns a slice of all registered providers.
func (r *ProviderRegistry) GetAllProviders() []PaymentGateway {
	var all []PaymentGateway
	for _, provider := range r.providers {
		all = append(all, provider)
	}
	return all
}
