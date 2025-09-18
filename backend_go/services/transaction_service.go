package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
)

type TransactionService interface {
	GetAll() ([]models.TransactionResponse, error)
}

type transactionService struct {
	transactionRepo repositories.TransactionRepository
}

func NewTransactionService(transactionRepo repositories.TransactionRepository) TransactionService {
	return &transactionService{transactionRepo: transactionRepo}
}

func (s *transactionService) GetAll() ([]models.TransactionResponse, error) {
	transactions, err := s.transactionRepo.GetAll()
	if err != nil {
		return nil, err
	}

	var response []models.TransactionResponse
	for _, t := range transactions {
		response = append(response, models.TransactionResponse(t))
	}

	return response, nil
}
