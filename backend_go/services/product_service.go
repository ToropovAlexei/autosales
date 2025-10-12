package services

import (
	"database/sql"
	"fmt"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/external_providers"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"log/slog"
	"time"

	"github.com/gin-gonic/gin"
)

// ProductUpdatePayload is the DTO for partial product updates.
type ProductUpdatePayload struct {
	Name                   *string  `json:"name"`
	CategoryID             *uint    `json:"category_id"`
	Price                  *float64 `json:"price" binding:"omitempty,gte=0"`
	Type                   *string  `json:"type" binding:"omitempty,oneof=item subscription"`
	SubscriptionPeriodDays *int     `json:"subscription_period_days" binding:"omitempty,gte=0"`
	Stock                  *int     `json:"stock" binding:"omitempty,gte=0"`
}

type ProductService interface {
	GetProducts(categoryIDs []uint) ([]models.ProductResponse, error)
	GetProduct(id uint) (*models.ProductResponse, error)
	CreateProduct(ctx *gin.Context, name string, categoryID uint, price float64, initialStock int, productType string, subscriptionPeriodDays int) (*models.ProductResponse, error)
	UpdateProduct(ctx *gin.Context, id uint, payload ProductUpdatePayload) (*models.ProductResponse, error)
	DeleteProduct(ctx *gin.Context, id uint) error
	CreateStockMovement(ctx *gin.Context, productID uint, movementType models.StockMovementType, quantity int, description string, orderID *uint) (*models.StockMovement, error)
	SyncExternalProductsAndCategories() error
}

type productService struct {
	productRepo      repositories.ProductRepository
	categoryRepo     repositories.CategoryRepository
	providerRegistry *external_providers.ProviderRegistry
	auditLogService  AuditLogService
}

func NewProductService(productRepo repositories.ProductRepository, categoryRepo repositories.CategoryRepository, providerRegistry *external_providers.ProviderRegistry, auditLogService AuditLogService) ProductService {
	return &productService{productRepo: productRepo, categoryRepo: categoryRepo, providerRegistry: providerRegistry, auditLogService: auditLogService}
}

func (s *productService) UpdateProduct(ctx *gin.Context, id uint, payload ProductUpdatePayload) (*models.ProductResponse, error) {
	before, err := s.GetProduct(id)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Resource: "Product", ID: id}
	}

	product, err := s.productRepo.GetProductByID(id)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Resource: "Product", ID: id}
	}

	// Handle stock adjustment first
	if payload.Stock != nil && product.Type == "item" {
		currentStock, err := s.productRepo.GetStockForProduct(id)
		if err != nil {
			return nil, fmt.Errorf("failed to get current stock for product %d: %w", id, err)
		}

		difference := *payload.Stock - currentStock
		if difference != 0 {
			movement := &models.StockMovement{
				ProductID:   id,
				Type:        models.Adjustment,
				Quantity:    difference,
				Description: "Manual stock adjustment",
				CreatedAt:   time.Now().UTC(),
			}
			if err := s.productRepo.CreateStockMovement(movement); err != nil {
				return nil, fmt.Errorf("failed to create stock adjustment movement: %w", err)
			}
		}
	}

	// Handle other field updates
	updateMap := make(map[string]interface{})
	if payload.Name != nil {
		updateMap["name"] = *payload.Name
	}
	if payload.CategoryID != nil {
		if _, err := s.productRepo.FindCategoryByID(*payload.CategoryID); err != nil {
			return nil, &apperrors.ErrNotFound{Resource: "Category", ID: *payload.CategoryID}
		}
		updateMap["category_id"] = *payload.CategoryID
	}
	if payload.Price != nil {
		updateMap["price"] = *payload.Price
	}
	if payload.Type != nil {
		updateMap["type"] = *payload.Type
	}
	if payload.SubscriptionPeriodDays != nil {
		updateMap["subscription_period_days"] = *payload.SubscriptionPeriodDays
	}

	if len(updateMap) > 0 {
		if err := s.productRepo.UpdateProduct(product, updateMap); err != nil {
			return nil, err
		}
	}

	after, err := s.GetProduct(id)
	if err != nil {
		return nil, err
	}

	s.auditLogService.Log(ctx, "PRODUCT_UPDATE", "Product", id, map[string]interface{}{"before": before, "after": after})

	return after, nil
}

// ... (The rest of the service file)

func (s *productService) SyncExternalProductsAndCategories() error {
	providers := s.providerRegistry.GetAllProviders()
	for _, provider := range providers {
		externalProducts, err := provider.GetProducts()
		if err != nil {
			slog.Error("failed to get products from provider", "provider", provider.GetName(), "error", err)
			continue // Skip this provider if it fails
		}

		for _, p := range externalProducts {
			if len(p.Category) > 0 {
				_, err := s.categoryRepo.FindOrCreateByPath(p.Category)
				if err != nil {
					slog.Error("failed to find or create category path for external product", "provider", provider.GetName(), "path", p.Category, "error", err)
				}
			}
		}
	}
	return nil
}

func (s *productService) GetProducts(categoryIDs []uint) ([]models.ProductResponse, error) {
	if err := s.SyncExternalProductsAndCategories(); err != nil {
		slog.Error("failed to sync external products and categories", "error", err)
	}

	internalProducts, err := s.productRepo.GetProducts(categoryIDs)
	if err != nil {
		return nil, err
	}

	var response []models.ProductResponse
	for _, p := range internalProducts {
		stock := 0
		if p.Type == "item" {
			stock, err = s.productRepo.GetStockForProduct(p.ID)
			if err != nil {
				return nil, err
			}
		} else {
			stock = -1
		}

		response = append(response, models.ProductResponse{
			ID:                     p.ID,
			Name:                   p.Name,
			Price:                  p.Price,
			CategoryID:             p.CategoryID,
			Stock:                  stock,
			Type:                   p.Type,
			SubscriptionPeriodDays: p.SubscriptionPeriodDays,
		})
	}

	providers := s.providerRegistry.GetAllProviders()
	for _, provider := range providers {
		externalProducts, err := provider.GetProducts()
		if err != nil {
			continue
		}

		for _, p := range externalProducts {
			var categoryID uint
			if len(p.Category) > 0 {
				category, err := s.categoryRepo.FindOrCreateByPath(p.Category)
				if err == nil && category != nil {
					categoryID = category.ID
				}
			}

			if len(categoryIDs) > 0 {
				match := false
				for _, catID := range categoryIDs {
					if catID == categoryID {
						match = true
						break
					}
				}
				if !match {
					continue
				}
			}

			response = append(response, models.ProductResponse{
				Name:                   p.Name,
				Price:                  p.Price,
				CategoryID:             categoryID,
				Stock:                  -1,
				Type:                   "subscription",
				SubscriptionPeriodDays: 30,
				Provider:               provider.GetName(),
				ExternalID:             p.ExternalID,
			})
		}
	}

	return response, nil
}

func (s *productService) GetProduct(id uint) (*models.ProductResponse, error) {
	product, err := s.productRepo.GetProductByID(id)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Resource: "Product", ID: id}
	}

	stock := 0
	if product.Type == "item" {
		stock, err = s.productRepo.GetStockForProduct(product.ID)
		if err != nil {
			return nil, err
		}
	} else {
		stock = -1
	}

	return &models.ProductResponse{
		ID:                     product.ID,
		Name:                   product.Name,
		Price:                  product.Price,
		CategoryID:             product.CategoryID,
		Stock:                  stock,
		Type:                   product.Type,
		SubscriptionPeriodDays: product.SubscriptionPeriodDays,
	}, nil
}

func (s *productService) CreateProduct(ctx *gin.Context, name string, categoryID uint, price float64, initialStock int, productType string, subscriptionPeriodDays int) (*models.ProductResponse, error) {
	_, err := s.productRepo.FindCategoryByID(categoryID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Resource: "Category", ID: categoryID}
	}

	product := &models.Product{
		Name:                   name,
		CategoryID:             categoryID,
		Price:                  price,
		Type:                   productType,
		SubscriptionPeriodDays: subscriptionPeriodDays,
		Details:                sql.NullString{String: "{}", Valid: true},
	}
	if err := s.productRepo.CreateProduct(product); err != nil {
		return nil, err
	}

	stock := 0
	if product.Type == "item" {
		stockMovement := &models.StockMovement{
			ProductID:   product.ID,
			Type:        models.Initial,
			Quantity:    initialStock,
			Description: "Initial stock",
			CreatedAt:   time.Now().UTC(),
		}
		if err := s.productRepo.CreateStockMovement(stockMovement); err != nil {
			// TODO: Should we roll back product creation?
			return nil, err
		}
		stock = initialStock
	} else {
		stock = -1
	}

	response := &models.ProductResponse{
		ID:                     product.ID,
		Name:                   product.Name,
		Price:                  product.Price,
		CategoryID:             product.CategoryID,
		Stock:                  stock,
		Type:                   product.Type,
		SubscriptionPeriodDays: product.SubscriptionPeriodDays,
	}

	s.auditLogService.Log(ctx, "PRODUCT_CREATE", "Product", product.ID, map[string]interface{}{"after": response})

	return response, nil
}

func (s *productService) DeleteProduct(ctx *gin.Context, id uint) error {
	before, err := s.GetProduct(id)
	if err != nil {
		return &apperrors.ErrNotFound{Resource: "Product", ID: id}
	}

	product, err := s.productRepo.GetProductByID(id)
	if err != nil {
		return &apperrors.ErrNotFound{Resource: "Product", ID: id}
	}

	if err := s.productRepo.DeleteProduct(product); err != nil {
		return err
	}

	s.auditLogService.Log(ctx, "PRODUCT_DELETE", "Product", id, map[string]interface{}{"before": before})

	return nil
}

func (s *productService) CreateStockMovement(ctx *gin.Context, productID uint, movementType models.StockMovementType, quantity int, description string, orderID *uint) (*models.StockMovement, error) {
	product, err := s.productRepo.GetProductByID(productID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Resource: "Product", ID: productID}
	}

	if product.Type != "item" {
		return nil, apperrors.New(400, "stock movements are not applicable to subscription products", nil)
	}

	movement := &models.StockMovement{
		ProductID:   productID,
		Type:        movementType,
		Quantity:    quantity,
		Description: description,
		OrderID:     orderID,
		CreatedAt:   time.Now().UTC(),
	}

	if err := s.productRepo.CreateStockMovement(movement); err != nil {
		return nil, err
	}

	s.auditLogService.Log(ctx, "STOCK_MOVEMENT_CREATE", "StockMovement", movement.ID, map[string]interface{}{"after": movement})

	return movement, nil
}
