package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type BalanceRepository interface {
	WithTx(tx *gorm.DB) BalanceRepository
	CreateDepositTransaction(transaction *models.Transaction) error
}

type gormBalanceRepository struct {
	db *gorm.DB
}

func NewBalanceRepository(db *gorm.DB) BalanceRepository {
	return &gormBalanceRepository{db: db}
}

func (r *gormBalanceRepository) WithTx(tx *gorm.DB) BalanceRepository {
	return &gormBalanceRepository{db: tx}
}

func (r *gormBalanceRepository) CreateDepositTransaction(transaction *models.Transaction) error {
	return r.db.Create(transaction).Error
}
