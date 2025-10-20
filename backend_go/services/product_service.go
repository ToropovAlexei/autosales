package services

import (
	"database/sql"
	"fmt"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/external_providers"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"log/slog"
	"reflect"
	"sort"
	"strconv"
	"strings"
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
	FulfillmentType        *string  `json:"fulfillment_type"`
	FulfillmentContent     *string  `json:"fulfillment_content"`
}

type ProductService interface {
	GetProducts(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.ProductResponse], error)
	GetProductsForBot(categoryID uint) ([]models.ProductResponse, error)
	GetProduct(id uint) (*models.ProductResponse, error)
	CreateProduct(ctx *gin.Context, name string, categoryID uint, price float64, initialStock int, productType string, subscriptionPeriodDays int, fulfillmentType string, fulfillmentContent string) (*models.ProductResponse, error)
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

func (s *productService) GetProductsForBot(categoryID uint) ([]models.ProductResponse, error) {
	if err := s.SyncExternalProductsAndCategories(); err != nil {
		slog.Error("failed to sync external products and categories", "error", err)
	}

	var filters []models.Filter
	if categoryID != 0 {
		filters = append(filters, models.Filter{Field: "category_id", Operator: "=", Value: categoryID})
	}

	internalProducts, err := s.productRepo.GetProducts(filters)
	if err != nil {
		return nil, err
	}

	var allProducts []models.ProductResponse
	for _, p := range internalProducts {
		stock, err := s.productRepo.GetStockForProduct(p.ID)
		if err != nil {
			return nil, err
		}
		allProducts = append(allProducts, models.ProductResponse{
			ID:                     p.ID,
			Name:                   p.Name,
			Price:                  p.Price,
			CategoryID:             p.CategoryID,
			Stock:                  stock,
			Type:                   p.Type,
			SubscriptionPeriodDays: p.SubscriptionPeriodDays,
			Visible:                p.Visible,
			FulfillmentType:        p.FulfillmentType,
			FulfillmentContent:     p.FulfillmentContent,
		})
	}

	providers := s.providerRegistry.GetAllProviders()
	for _, provider := range providers {
		externalProducts, err := provider.GetProducts()
		if err != nil {
			slog.Error("failed to get products from provider", "provider", provider.GetName(), "error", err)
			continue
		}
		filteredExternal := s.filterExternalProducts(externalProducts, filters)

		for _, p := range filteredExternal {
			var pCategoryID uint
			if len(p.Category) > 0 {
				category, err := s.categoryRepo.FindOrCreateByPath(p.Category)
				if err == nil && category != nil {
					pCategoryID = category.ID
				}
			}
			allProducts = append(allProducts, models.ProductResponse{
				Name:                   p.Name,
				Price:                  p.Price,
				CategoryID:             pCategoryID,
				Stock:                  -1, // External products are subscriptions, stock is not applicable
				Type:                   "subscription",
				SubscriptionPeriodDays: 30, // Assuming a default, this could be part of the external product data
				Provider:               provider.GetName(),
				ExternalID:             p.ExternalID,
				Visible:                true, // Assuming external are always visible
			})
		}
	}

	return allProducts, nil
}

func (s *productService) GetProducts(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.ProductResponse], error) {
	if err := s.SyncExternalProductsAndCategories(); err != nil {
		slog.Error("failed to sync external products and categories", "error", err)
	}

	// 1. Get filtered internal products (without pagination)
	internalProducts, err := s.productRepo.GetProducts(filters)
	if err != nil {
		return nil, err
	}

	var allProducts []models.ProductResponse
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
		allProducts = append(allProducts, models.ProductResponse{
			ID:                     p.ID,
			Name:                   p.Name,
			Price:                  p.Price,
			CategoryID:             p.CategoryID,
			Stock:                  stock,
			Type:                   p.Type,
			SubscriptionPeriodDays: p.SubscriptionPeriodDays,
			Visible:                p.Visible,
			FulfillmentType:        p.FulfillmentType,
			FulfillmentContent:     p.FulfillmentContent,
		})
	}

	// 2. Get and filter external products
	providers := s.providerRegistry.GetAllProviders()
	for _, provider := range providers {
		externalProducts, err := provider.GetProducts()
		if err != nil {
			slog.Error("failed to get products from provider", "provider", provider.GetName(), "error", err)
			continue
		}

		// In-memory filtering for external products
		filteredExternal := s.filterExternalProducts(externalProducts, filters)

		for _, p := range filteredExternal {
			var categoryID uint
			if len(p.Category) > 0 {
				category, err := s.categoryRepo.FindOrCreateByPath(p.Category)
				if err == nil && category != nil {
					categoryID = category.ID
				}
			}
			allProducts = append(allProducts, models.ProductResponse{
				Name:                   p.Name,
				Price:                  p.Price,
				CategoryID:             categoryID,
				Stock:                  -1, // External products are subscriptions, stock is not applicable
				Type:                   "subscription",
				SubscriptionPeriodDays: 30, // Assuming a default, this could be part of the external product data
				Provider:               provider.GetName(),
				ExternalID:             p.ExternalID,
				Visible:                true, // Assuming external are always visible
			})
		}
	}

	// 3. Sort the combined list
	sort.SliceStable(allProducts, func(i, j int) bool {
		valI := reflect.ValueOf(allProducts[i])
		valJ := reflect.ValueOf(allProducts[j])

		// Use reflection to get field value by name
		fieldI := valI.FieldByNameFunc(func(s string) bool { return strings.EqualFold(s, page.OrderBy) })
		fieldJ := valJ.FieldByNameFunc(func(s string) bool { return strings.EqualFold(s, page.OrderBy) })

		if !fieldI.IsValid() || !fieldJ.IsValid() {
			return false // Cannot sort by this field
		}

		// Generic comparison for different types
		switch fieldI.Kind() {
		case reflect.String:
			if page.Order == "asc" {
				return fieldI.String() < fieldJ.String()
			}
			return fieldI.String() > fieldJ.String()
		case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
			if page.Order == "asc" {
				return fieldI.Int() < fieldJ.Int()
			}
			return fieldI.Int() > fieldJ.Int()
		case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
			// Special handling for ID, where external products have ID 0
			if page.OrderBy == "id" {
				idI, idJ := fieldI.Uint(), fieldJ.Uint()
				if idI == 0 {
					return false
				} // Push external to the end
				if idJ == 0 {
					return true
				}
			}
			if page.Order == "asc" {
				return fieldI.Uint() < fieldJ.Uint()
			}
			return fieldI.Uint() > fieldJ.Uint()
		case reflect.Float32, reflect.Float64:
			if page.Order == "asc" {
				return fieldI.Float() < fieldJ.Float()
			}
			return fieldI.Float() > fieldJ.Float()
		case reflect.Bool:
			if page.Order == "asc" {
				return !fieldI.Bool() && fieldJ.Bool()
			}
			return fieldI.Bool() && !fieldJ.Bool()
		default:
			return false
		}
	})

	// 4. Paginate the combined list
	total := int64(len(allProducts))
	start := (page.Page - 1) * page.PageSize
	if start > len(allProducts) {
		start = len(allProducts)
	}
	end := start + page.PageSize
	if end > len(allProducts) {
		end = len(allProducts)
	}

	paginatedData := allProducts[start:end]

	return &models.PaginatedResult[models.ProductResponse]{
		Data:  paginatedData,
		Total: total,
	}, nil
}

// filterExternalProducts applies filters to a slice of external products in memory.
func (s *productService) filterExternalProducts(products []external_providers.ProviderProduct, filters []models.Filter) []external_providers.ProviderProduct {
	if len(filters) == 0 {
		return products
	}

	var filtered []external_providers.ProviderProduct
	for _, p := range products {
		matches := true
		for _, f := range filters {
			var categoryID uint
			if len(p.Category) > 0 {
				category, err := s.categoryRepo.FindOrCreateByPath(p.Category)
				if err == nil && category != nil {
					categoryID = category.ID
				}
			}

			val := reflect.ValueOf(p)
			fieldVal := val.FieldByNameFunc(func(s string) bool { return strings.EqualFold(s, f.Field) })

			if f.Field == "category_id" {
				var filterCatID uint
				switch v := f.Value.(type) {
				case float64:
					filterCatID = uint(v)
				case uint:
					filterCatID = v
				default:
					matches = false
					break
				}

				if categoryID != filterCatID {
					matches = false
				}
				continue
			}

			if !fieldVal.IsValid() {
				matches = false
				break
			}

			if !matchFilter(fieldVal, f) {
				matches = false
				break
			}
		}
		if matches {
			filtered = append(filtered, p)
		}
	}
	return filtered
}

// matchFilter compares a reflected value with a filter.
func matchFilter(value reflect.Value, filter models.Filter) bool {
	op := strings.ToLower(filter.Operator)

	switch value.Kind() {
	case reflect.String:
		filterValue, ok := filter.Value.(string)
		if !ok {
			return false
		}
		switch op {
		case "contains":
			return strings.Contains(strings.ToLower(value.String()), strings.ToLower(filterValue))
		case "=":
			return strings.EqualFold(value.String(), filterValue)
		default:
			return false
		}
	case reflect.Float64:
		filterValue, err := strconv.ParseFloat(fmt.Sprintf("%v", filter.Value), 64)
		if err != nil {
			return false
		}
		switch op {
		case "=":
			return value.Float() == filterValue
		case ">":
			return value.Float() > filterValue
		case "<":
			return value.Float() < filterValue
		case ">=":
			return value.Float() >= filterValue
		case "<=":
			return value.Float() <= filterValue
		default:
			return false
		}
	case reflect.Uint, reflect.Uint64:
		// This handles category_id
		filterValue, ok := filter.Value.(float64) // JSON numbers are float64
		if !ok {
			return false
		}
		switch op {
		case "=":
			return value.Uint() == uint64(filterValue)
		// "in" operator for category_id
		case "in":
			filterValues, ok := filter.Value.([]interface{})
			if !ok {
				return false
			}
			for _, v := range filterValues {
				valFloat, ok := v.(float64)
				if ok && value.Uint() == uint64(valFloat) {
					return true
				}
			}
			return false
		default:
			return false
		}
	case reflect.Bool:
		filterValue, ok := filter.Value.(bool)
		if !ok {
			return false
		}
		if op == "=" {
			return value.Bool() == filterValue
		}
		return false
	default:
		return false
	}
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

func (s *productService) CreateProduct(ctx *gin.Context, name string, categoryID uint, price float64, initialStock int, productType string, subscriptionPeriodDays int, fulfillmentType string, fulfillmentContent string) (*models.ProductResponse, error) {
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
		FulfillmentType:        fulfillmentType,
		FulfillmentContent:     fulfillmentContent,
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
	if payload.FulfillmentType != nil {
		updateMap["fulfillment_type"] = *payload.FulfillmentType
	}
	if payload.FulfillmentContent != nil {
		updateMap["fulfillment_content"] = *payload.FulfillmentContent
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
