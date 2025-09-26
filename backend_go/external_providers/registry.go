package external_providers

import "fmt"

// ProviderRegistry holds a map of all registered external product providers.
type ProviderRegistry struct {
	providers map[string]ExternalProductProvider
}

// NewProviderRegistry creates a new, empty provider registry.
func NewProviderRegistry() *ProviderRegistry {
	return &ProviderRegistry{
		providers: make(map[string]ExternalProductProvider),
	}
}

// RegisterProvider adds a provider to the registry.
func (r *ProviderRegistry) RegisterProvider(provider ExternalProductProvider) {
	r.providers[provider.GetName()] = provider
}

// GetProvider retrieves a provider by its name.
func (r *ProviderRegistry) GetProvider(name string) (ExternalProductProvider, error) {
	provider, ok := r.providers[name]
	if !ok {
		return nil, fmt.Errorf("provider with name '%s' not found", name)
	}
	return provider, nil
}

// GetAllProviders returns all registered providers.
func (r *ProviderRegistry) GetAllProviders() []ExternalProductProvider {
	var providers []ExternalProductProvider
	for _, p := range r.providers {
		providers = append(providers, p)
	}
	return providers
}
