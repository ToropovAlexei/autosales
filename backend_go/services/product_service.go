package services

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"time"
)

type ProductService interface {
	GetProducts(categoryIDs []string) ([]models.ProductResponse, error)
	GetProduct(id uint) (*models.ProductResponse, error)
	CreateProduct(name string, categoryID uint, price float64, initialStock int, productType string, subscriptionPeriodDays int) (*models.ProductResponse, error)
	UpdateProduct(id uint, data models.Product) (*models.ProductResponse, error)
	DeleteProduct(id uint) error
	CreateStockMovement(productID uint, movementType models.StockMovementType, quantity int, description string, orderID *uint) (*models.StockMovement, error)
}

type productService struct {
	productRepo repositories.ProductRepository
}

func NewProductService(productRepo repositories.ProductRepository) ProductService {
	return &productService{productRepo: productRepo}
}

func (s *productService) GetProducts(categoryIDs []string) ([]models.ProductResponse, error) {
	products, err := s.productRepo.GetProducts(categoryIDs)
	if err != nil {
		return nil, err
	}

	var response []models.ProductResponse
	for _, p := range products {
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
