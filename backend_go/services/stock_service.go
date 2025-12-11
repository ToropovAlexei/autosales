package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
)

type StockService interface {
	GetStockMovements(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.StockMovementResponse], error)
}

type stockService struct {
	stockRepo repositories.StockRepository
}

func NewStockService(stockRepo repositories.StockRepository) StockService {
	return &stockService{stockRepo: stockRepo}
}

func (s *stockService) GetStockMovements(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.StockMovementResponse], error) {
	paginatedMovements, err := s.stockRepo.GetStockMovements(page, filters)
	if err != nil {
		return nil, err
	}

	var response []models.StockMovementResponse
	for _, m := range paginatedMovements.Data {
		response = append(response, models.StockMovementResponse{
			ID:          m.ID,
			ProductID:   m.ProductID,
			ProductName: m.Product.Name,
			Type:        m.Type,
			Quantity:    m.Quantity,
			CreatedAt:   m.CreatedAt,
			Description: m.Description,
			OrderID:     m.OrderID,
		})
	}

	return &models.PaginatedResult[models.StockMovementResponse]{
		Data:  response,
		Total: paginatedMovements.Total,
	}, nil
}
