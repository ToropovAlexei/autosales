package services

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/external_providers"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"log/slog"
	"time"
)

type ProductService interface {
	GetProducts(categoryIDs []uint) ([]models.ProductResponse, error)
	GetProduct(id uint) (*models.ProductResponse, error)
	CreateProduct(name string, categoryID uint, price float64, initialStock int, productType string, subscriptionPeriodDays int) (*models.ProductResponse, error)
	UpdateProduct(id uint, data models.Product) (*models.ProductResponse, error)
	DeleteProduct(id uint) error
	CreateStockMovement(productID uint, movementType models.StockMovementType, quantity int, description string, orderID *uint) (*models.StockMovement, error)
}

type productService struct {
	productRepo      repositories.ProductRepository
	categoryRepo     repositories.CategoryRepository
	providerRegistry *external_providers.ProviderRegistry
}

func NewProductService(productRepo repositories.ProductRepository, categoryRepo repositories.CategoryRepository, providerRegistry *external_providers.ProviderRegistry) ProductService {
	return &productService{productRepo: productRepo, categoryRepo: categoryRepo, providerRegistry: providerRegistry}
}

func (s *productService) GetProducts(categoryIDs []uint) ([]models.ProductResponse, error) {
	// 1. Get internal products
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
			stock = -1 // Infinite stock for subscriptions
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

	// 2. Get external products
	providers := s.providerRegistry.GetAllProviders()
	for _, provider := range providers {
		externalProducts, err := provider.GetProducts()
		if err != nil {
			slog.Error("failed to get products from provider", "provider", provider.GetName(), "error", err)
			continue // Skip this provider if it fails
		}

		for _, p := range externalProducts {
			var categoryID uint
			if len(p.Category) > 0 {
				category, err := s.categoryRepo.FindOrCreateByPath(p.Category)
				if err != nil {
					slog.Error("failed to find or create category path for external product", "provider", provider.GetName(), "path", p.Category, "error", err)
				} else if category != nil {
					categoryID = category.ID
				}
			}

			// Filter by category if categoryIDs are provided
			if len(categoryIDs) > 0 {
				match := false
				for _, catID := range categoryIDs {
					if catID == categoryID {
						match = true
						break
					}
				}
				if !match {
					continue // Skip product if it doesn't match the category filter
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
	// This now only works for internal products.
	// To support external products, we would need to change the ID system or the method signature.
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
		stock = -1 // Infinite stock for subscriptions
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

func (s *productService) CreateProduct(name string, categoryID uint, price float64, initialStock int, productType string, subscriptionPeriodDays int) (*models.ProductResponse, error) {
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
			// Here you might want to consider rolling back the product creation
			return nil, err
		}
		stock = initialStock
	} else {
		stock = -1 // Infinite stock for subscriptions
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

func (s *productService) UpdateProduct(id uint, data models.Product) (*models.ProductResponse, error) {
	product, err := s.productRepo.GetProductByID(id)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Resource: "Product", ID: id}
	}

	if _, err := s.productRepo.FindCategoryByID(data.CategoryID); err != nil {
		return nil, &apperrors.ErrNotFound{Resource: "Category", ID: data.CategoryID}
	}

	if err := s.productRepo.UpdateProduct(product, data); err != nil {
		return nil, err
	}

	return s.GetProduct(id)
}

func (s *productService) DeleteProduct(id uint) error {
	product, err := s.productRepo.GetProductByID(id)
	if err != nil {
		return &apperrors.ErrNotFound{Resource: "Product", ID: id}
	}
	return s.productRepo.DeleteProduct(product)
}

func (s *productService) CreateStockMovement(productID uint, movementType models.StockMovementType, quantity int, description string, orderID *uint) (*models.StockMovement, error) {
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

	return movement, nil
}
