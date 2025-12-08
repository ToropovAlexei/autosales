package workers

import (
	"database/sql"
	"fmt"
	"frbktg/backend_go/external_providers"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"log/slog"
	"time"
)

type ProductSyncWorker struct {
	productRepo      repositories.ProductRepository
	categoryRepo     repositories.CategoryRepository
	providerRegistry *external_providers.ProviderRegistry
}

func NewProductSyncWorker(productRepo repositories.ProductRepository, categoryRepo repositories.CategoryRepository, providerRegistry *external_providers.ProviderRegistry) *ProductSyncWorker {
	return &ProductSyncWorker{
		productRepo:      productRepo,
		categoryRepo:     categoryRepo,
		providerRegistry: providerRegistry,
	}
}

func (w *ProductSyncWorker) Run() {
	slog.Info("Starting product sync worker...")
	ticker := time.NewTicker(5 * time.Minute)
	defer ticker.Stop()

	// Run once on start
	if err := w.syncProducts(); err != nil {
		slog.Error("Error during initial product sync", "error", err)
	}

	for range ticker.C {
		slog.Info("Running product sync...")
		if err := w.syncProducts(); err != nil {
			slog.Error("Error during product sync", "error", err)
		}
	}
}

func (w *ProductSyncWorker) Start() {
	go w.Run()
}

func (w *ProductSyncWorker) syncProducts() error {
	providers := w.providerRegistry.GetAllProviders()
	localExternalProducts, err := w.productRepo.GetExternalProducts()
	if err != nil {
		return fmt.Errorf("failed to get local external products: %w", err)
	}

	localProductMap := make(map[string]models.Product)
	for _, p := range localExternalProducts {
		key := fmt.Sprintf("%s-%s", *p.ProviderName, *p.ExternalID)
		localProductMap[key] = p
	}

	for _, provider := range providers {
		externalProducts, err := provider.GetProducts()
		if err != nil {
			slog.Error("failed to get products from provider", "provider", provider.GetName(), "error", err)
			continue
		}

		for _, externalProduct := range externalProducts {
			key := fmt.Sprintf("%s-%s", provider.GetName(), externalProduct.ExternalID)
			category, err := w.categoryRepo.FindOrCreateByPath(externalProduct.Category)
			if err != nil {
				slog.Error("failed to find or create category for external product", "path", externalProduct.Category, "error", err)
				continue
			}

			if localProduct, exists := localProductMap[key]; exists {
				// Product exists, check for updates
				updateMap := make(map[string]interface{})
				if localProduct.Name != externalProduct.Name {
					updateMap["name"] = externalProduct.Name
				}
				if localProduct.Price != externalProduct.Price {
					updateMap["price"] = externalProduct.Price
				}
				if localProduct.CategoryID != category.ID {
					updateMap["category_id"] = category.ID
				}
				if !localProduct.Visible {
					updateMap["visible"] = true // Re-enable if it was soft-deleted
				}
				if externalProduct.Host != "" {
					fulfillmentText := fmt.Sprintf("Host: %s\nPort: %d", externalProduct.Host, externalProduct.Port)
					updateMap["fulfillment_text"] = sql.NullString{String: fulfillmentText, Valid: true}
				}

				if len(updateMap) > 0 {
					if err := w.productRepo.UpdateProduct(&localProduct, updateMap); err != nil {
						slog.Error("failed to update external product", "key", key, "error", err)
					}
				}
				delete(localProductMap, key) // Remove from map to track which products to soft-delete
			} else {
				// Product does not exist, create it
				providerName := provider.GetName()
				newProduct := &models.Product{
					Name:         externalProduct.Name,
					Price:        externalProduct.Price,
					CategoryID:   category.ID,
					Type:         externalProduct.Type,
					ProviderName: &providerName,
					ExternalID:   &externalProduct.ExternalID,
					Visible:      true,
				}
				if externalProduct.Host != "" {
					fulfillmentText := fmt.Sprintf("Host: %s\nPort: %d", externalProduct.Host, externalProduct.Port)
					newProduct.FulfillmentText = sql.NullString{String: fulfillmentText, Valid: true}
				}
				if err := w.productRepo.CreateProduct(newProduct); err != nil {
					slog.Error("failed to create external product", "key", key, "error", err)
				}
			}
		}
	}

	// Soft-delete any remaining local products that were not in the provider's response
	for key, product := range localProductMap {
		if product.Visible {
			updateMap := map[string]interface{}{"visible": false}
			if err := w.productRepo.UpdateProduct(&product, updateMap); err != nil {
				slog.Error("failed to soft-delete external product", "key", key, "error", err)
			}
		}
	}

	return nil
}
