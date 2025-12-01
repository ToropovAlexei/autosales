package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"gorm.io/gorm"
)

type StoreBalanceService interface {
	GetStoreBalance() (*models.StoreBalance, error)
	UpdateStoreBalance(tx *gorm.DB, amount float64) error
	SetStoreBalance(tx *gorm.DB, amount float64) error
}

type storeBalanceService struct {
	repo repositories.StoreBalanceRepository
}

func NewStoreBalanceService(repo repositories.StoreBalanceRepository) StoreBalanceService {
	return &storeBalanceService{repo: repo}
}

func (s *storeBalanceService) GetStoreBalance() (*models.StoreBalance, error) {
	return s.repo.GetStoreBalance(nil)
}

func (s *storeBalanceService) UpdateStoreBalance(tx *gorm.DB, amount float64) error {
	return s.repo.UpdateStoreBalance(tx, amount)
}

func (s *storeBalanceService) SetStoreBalance(tx *gorm.DB, amount float64) error {
	return s.repo.SetStoreBalance(tx, amount)
}
