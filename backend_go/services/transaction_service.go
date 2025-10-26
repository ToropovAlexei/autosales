package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
)

type TransactionService interface {
	GetAll(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.TransactionResponse], error)
}

type transactionService struct {
	transactionRepo repositories.TransactionRepository
}

func NewTransactionService(transactionRepo repositories.TransactionRepository) TransactionService {
	return &transactionService{transactionRepo: transactionRepo}
}

func (s *transactionService) GetAll(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.TransactionResponse], error) {
	paginatedTransactions, err := s.transactionRepo.GetAll(page, filters)
	if err != nil {
		return nil, err
	}

	var response []models.TransactionResponse
	for _, t := range paginatedTransactions.Data {
		response = append(response, models.TransactionResponse{
			ID:          t.ID,
			UserID:      t.UserID,
			OrderID:     t.OrderID,
			Type:        t.Type,
			Amount:      t.Amount,
			CreatedAt:   t.CreatedAt,
			Description: t.Description,
		})
	}

	return &models.PaginatedResult[models.TransactionResponse]{
		Data:  response,
		Total: paginatedTransactions.Total,
	}, nil
}
