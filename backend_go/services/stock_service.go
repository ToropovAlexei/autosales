package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
)

type StockService interface {
	GetStockMovements() ([]models.StockMovementResponse, error)
}

type stockService struct {
	stockRepo repositories.StockRepository
}

func NewStockService(stockRepo repositories.StockRepository) StockService {
	return &stockService{stockRepo: stockRepo}
}

func (s *stockService) GetStockMovements() ([]models.StockMovementResponse, error) {
	movements, err := s.stockRepo.GetStockMovements()
	if err != nil {
		return nil, err
	}

	var response []models.StockMovementResponse
	for _, m := range movements {
		response = append(response, models.StockMovementResponse{
			ID:          m.ID,
			ProductID:   m.ProductID,
			Type:        m.Type,
			Quantity:    m.Quantity,
			CreatedAt:   m.CreatedAt,
			Description: m.Description,
			OrderID:     m.OrderID,
		})
	}

	return response, nil
}
