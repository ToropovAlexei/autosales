package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"gorm.io/gorm"
)

type StoreBalanceService interface {
	GetStoreBalance() (*models.StoreBalance, error)
	UpdateStoreBalance(tx *gorm.DB, amount float64) error
}

type storeBalanceService struct {
	repo repositories.StoreBalanceRepository
	db   *gorm.DB
}

func NewStoreBalanceService(repo repositories.StoreBalanceRepository, db *gorm.DB) StoreBalanceService {
	return &storeBalanceService{repo: repo, db: db}
}

func (s *storeBalanceService) GetStoreBalance() (*models.StoreBalance, error) {
	return s.repo.GetStoreBalance(nil)
}

func (s *storeBalanceService) UpdateStoreBalance(tx *gorm.DB, amount float64) error {
	return s.repo.UpdateStoreBalance(tx, amount)
}
