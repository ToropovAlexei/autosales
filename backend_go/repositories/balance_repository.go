package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type BalanceRepository interface {
	CreateDepositTransaction(transaction *models.Transaction) error
}

type gormBalanceRepository struct {
	db *gorm.DB
}

func NewBalanceRepository(db *gorm.DB) BalanceRepository {
	return &gormBalanceRepository{db: db}
}

func (r *gormBalanceRepository) CreateDepositTransaction(transaction *models.Transaction) error {
	return r.db.Create(transaction).Error
}
