package services

import (
	"database/sql"
	"encoding/csv"
	"fmt"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/external_providers"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"io"
	"log/slog"
	"math"
	"reflect"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
)

// ProductUpdatePayload is the DTO for partial product updates.
type ProductUpdatePayload struct {
	Name                   *string    `json:"name"`
	CategoryID             *uint      `json:"category_id"`
	BasePrice              *float64   `json:"base_price" binding:"omitempty,gte=0"`
	ImageID                *uuid.UUID `json:"image_id"`
	Type                   *string    `json:"type" binding:"omitempty,oneof=item subscription"`
	SubscriptionPeriodDays *int       `json:"subscription_period_days" binding:"omitempty,gte=0"`
	Stock                  *int       `json:"stock" binding:"omitempty,gte=0"`
	FulfillmentType        *string    `json:"fulfillment_type"`
	FulfillmentContent     *string    `json:"fulfillment_content"`
}

type ProductService interface {
	GetProducts(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.ProductResponse], error)
	GetProductsForBot(categoryID uint) ([]models.ProductResponse, error)
	GetProduct(id uint, gateway string) (*models.ProductResponse, error)
	GetProductForBot(id uint) (*models.ProductResponse, error)
	CreateProduct(ctx *gin.Context, name string, categoryID uint, basePrice float64, initialStock int, productType string, subscriptionPeriodDays int, fulfillmentType string, fulfillmentContent string, imageID *uuid.UUID) (*models.ProductResponse, error)
	UpdateProduct(ctx *gin.Context, id uint, payload ProductUpdatePayload) (*models.ProductResponse, error)
	DeleteProduct(ctx *gin.Context, id uint) error
	CreateStockMovement(ctx *gin.Context, productID uint, movementType models.StockMovementType, quantity int, description string, orderID *uint) (*models.StockMovement, error)
	UploadProductsCSV(ctx *gin.Context, file io.Reader) (map[string]interface{}, error)
}

type productService struct {
	productRepo      repositories.ProductRepository
	categoryRepo     repositories.CategoryRepository
	providerRegistry *external_providers.ProviderRegistry
	auditLogService  AuditLogService
	settingService   SettingService
}

func NewProductService(productRepo repositories.ProductRepository, categoryRepo repositories.CategoryRepository, providerRegistry *external_providers.ProviderRegistry, auditLogService AuditLogService, settingService SettingService) ProductService {
	return &productService{productRepo: productRepo, categoryRepo: categoryRepo, providerRegistry: providerRegistry, auditLogService: auditLogService, settingService: settingService}
}

func (s *productService) calculatePrice(basePrice float64, gateway string, settings map[string]string) float64 {
	price := basePrice

	// Apply global markup
	if markupStr, ok := settings["GLOBAL_PRICE_MARKUP"]; ok {
		markup, err := strconv.ParseFloat(markupStr, 64)
		if err == nil {
			price = price * (1 + markup/100)
		}
	}

	// Apply payment system markup
	if markupStr, ok := settings["PAYMENT_SYSTEM_MARKUP"]; ok {
		markup, err := strconv.ParseFloat(markupStr, 64)
		if err == nil {
			price = price / (1 - markup/100)
		}
	}

	return math.Round(price)
}

func (s *productService) GetProductsForBot(categoryID uint) ([]models.ProductResponse, error) {
	var filters []models.Filter
	if categoryID != 0 {
		filters = append(filters, models.Filter{Field: "category_id", Operator: "=", Value: categoryID})
	}

	products, err := s.productRepo.GetProducts(filters)
	if err != nil {
		return nil, err
	}

	settings, err := s.settingService.GetSettings()
	if err != nil {
		return nil, apperrors.New(500, "failed to get settings", err)
	}

	var response = make([]models.ProductResponse, 0)
	for _, p := range products {
		stock, err := s.productRepo.GetStockForProduct(p.ID)
		if err != nil {
			return nil, err
		}

		providerName := ""
		if p.ProviderName != nil {
			providerName = *p.ProviderName
		}

		externalID := ""
		if p.ExternalID != nil {
			externalID = *p.ExternalID
		}

		var imageIDStr string
		if p.ImageID != nil {
			imageIDStr = p.ImageID.String()
		}

		finalPrice := s.calculatePrice(p.Price, "", settings)

		response = append(response, models.ProductResponse{
			ID:                     p.ID,
			Name:                   p.Name,
			BasePrice:              p.Price,
			Price:                  finalPrice,
			CategoryID:             p.CategoryID,
			Stock:                  stock,
			Type:                   p.Type,
			SubscriptionPeriodDays: p.SubscriptionPeriodDays,
			Visible:                p.Visible,
			FulfillmentType:        p.FulfillmentType,
			FulfillmentContent:     p.FulfillmentContent,
			ImageID:                imageIDStr,
			Provider:               providerName,
			ExternalID:             externalID,
		})
	}

	return response, nil
}

func (s *productService) GetProducts(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.ProductResponse], error) {
	products, err := s.productRepo.GetProducts(filters)
	if err != nil {
		return nil, err
	}

	settings, err := s.settingService.GetSettings()
	if err != nil {
		return nil, apperrors.New(500, "failed to get settings", err)
	}

	var allProducts []models.ProductResponse
	for _, p := range products {
		stock := 0
		if p.Type == "item" {
			stock, err = s.productRepo.GetStockForProduct(p.ID)
			if err != nil {
				return nil, err
			}
		} else {
			stock = -1
		}

		providerName := ""
		if p.ProviderName != nil {
			providerName = *p.ProviderName
		}

		externalID := ""
		if p.ExternalID != nil {
			externalID = *p.ExternalID
		}

		var imageIDStr string
		if p.ImageID != nil {
			imageIDStr = p.ImageID.String()
		}

		finalPrice := s.calculatePrice(p.Price, "", settings)

		allProducts = append(allProducts, models.ProductResponse{
			ID:                     p.ID,
			Name:                   p.Name,
			BasePrice:              p.Price,
			Price:                  finalPrice,
			CategoryID:             p.CategoryID,
			Stock:                  stock,
			Type:                   p.Type,
			SubscriptionPeriodDays: p.SubscriptionPeriodDays,
			Visible:                p.Visible,
			FulfillmentType:        p.FulfillmentType,
			FulfillmentContent:     p.FulfillmentContent,
			ImageID:                imageIDStr,
			Provider:               providerName,
			ExternalID:             externalID,
		})
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

func (s *productService) GetProduct(id uint, gateway string) (*models.ProductResponse, error) {
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

	var imageIDStr string
	if product.ImageID != nil {
		imageIDStr = product.ImageID.String()
	}

	settings, err := s.settingService.GetSettings()
	if err != nil {
		return nil, apperrors.New(500, "failed to get settings", err)
	}

	finalPrice := s.calculatePrice(product.Price, gateway, settings)

	return &models.ProductResponse{
		ID:                     product.ID,
		Name:                   product.Name,
		BasePrice:              product.Price,
		Price:                  finalPrice,
		CategoryID:             product.CategoryID,
		Stock:                  stock,
		Type:                   product.Type,
		SubscriptionPeriodDays: product.SubscriptionPeriodDays,
		ImageID:                imageIDStr,
	}, nil
}

func (s *productService) GetProductForBot(id uint) (*models.ProductResponse, error) {
	return s.GetProduct(id, "")
}

func (s *productService) CreateProduct(ctx *gin.Context, name string, categoryID uint, basePrice float64, initialStock int, productType string, subscriptionPeriodDays int, fulfillmentType string, fulfillmentContent string, imageID *uuid.UUID) (*models.ProductResponse, error) {
	_, err := s.productRepo.FindCategoryByID(categoryID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Resource: "Category", ID: categoryID}
	}

	product := &models.Product{
		Name:                   name,
		CategoryID:             categoryID,
		Price:                  basePrice,
		Type:                   productType,
		SubscriptionPeriodDays: subscriptionPeriodDays,
		Details:                sql.NullString{String: "{}", Valid: true},
		FulfillmentType:        fulfillmentType,
		FulfillmentContent:     fulfillmentContent,
		ImageID:                imageID,
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

	var imageIDStr string
	if product.ImageID != nil {
		imageIDStr = product.ImageID.String()
	}

	response := &models.ProductResponse{
		ID:                     product.ID,
		Name:                   product.Name,
		BasePrice:              product.Price,
		Price:                  product.Price, // At creation, calculated price is same as base
		CategoryID:             product.CategoryID,
		Stock:                  stock,
		Type:                   product.Type,
		SubscriptionPeriodDays: product.SubscriptionPeriodDays,
		ImageID:                imageIDStr,
	}

	s.auditLogService.Log(ctx, "PRODUCT_CREATE", "Product", product.ID, map[string]interface{}{"after": response})

	return response, nil
}

func (s *productService) DeleteProduct(ctx *gin.Context, id uint) error {
	before, err := s.GetProduct(id, "")
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
	before, err := s.GetProduct(id, "")
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
	if payload.BasePrice != nil {
		updateMap["price"] = *payload.BasePrice
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
	if payload.ImageID != nil {
		updateMap["image_id"] = *payload.ImageID
	}

	if len(updateMap) > 0 {
		if err := s.productRepo.UpdateProduct(product, updateMap); err != nil {
			return nil, err
		}
	}

	after, err := s.GetProduct(id, "")
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

func (s *productService) UploadProductsCSV(ctx *gin.Context, file io.Reader) (map[string]interface{}, error) {
	reader := csv.NewReader(file)
	reader.LazyQuotes = true
	records, err := reader.ReadAll()
	if err != nil {
		return nil, &apperrors.ErrValidation{Message: "Failed to parse CSV file"}
	}

	var createdCount, errorCount, skippedCount int
	var errors []string

	// Skip header row
	for i, record := range records {
		if i == 0 {
			continue
		}

		if len(record) < 4 {
			errors = append(errors, fmt.Sprintf("Row %d: not enough columns", i+1))
			errorCount++
			continue
		}

		name := record[0]
		categoryPath := record[1]
		price, err := strconv.ParseFloat(record[2], 64)
		if err != nil {
			errors = append(errors, fmt.Sprintf("Row %d: invalid price '%s'", i+1, record[2]))
			errorCount++
			continue
		}

		initialStock, err := strconv.Atoi(record[3])
		if err != nil {
			errors = append(errors, fmt.Sprintf("Row %d: invalid initial stock '%s'", i+1, record[3]))
			errorCount++
			continue
		}

		existingProduct, err := s.productRepo.FindByName(name)
		if err != nil {
			errors = append(errors, fmt.Sprintf("Row %d: error checking for existing product '%s': %v", i+1, name, err))
			errorCount++
			continue
		}
		if existingProduct != nil {
			skippedCount++
			continue
		}

		category, err := s.categoryRepo.FindOrCreateByPath(strings.Split(categoryPath, "/"))
		if err != nil {
			errors = append(errors, fmt.Sprintf("Row %d: failed to find or create category '%s': %v", i+1, categoryPath, err))
			errorCount++
			continue
		}

		_, err = s.CreateProduct(ctx, name, category.ID, price, initialStock, "item", 0, "", "", nil)
		if err != nil {
			errors = append(errors, fmt.Sprintf("Row %d: failed to create product '%s': %v", i+1, name, err))
			errorCount++
			continue
		}

		createdCount++
	}

	result := map[string]interface{}{
		"created": createdCount,
		"failed":  errorCount,
		"skipped": skippedCount,
		"errors":  errors,
	}

	return result, nil
}
