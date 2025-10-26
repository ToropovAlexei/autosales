package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

// TransactionRepository handles database operations related to financial transactions.
// It is designed to be used within a single database transaction.
type TransactionRepository interface {
	WithTx(tx *gorm.DB) TransactionRepository
	CreateTransaction(transaction *models.Transaction) error
	CreateRefTransaction(refTransaction *models.RefTransaction) error
	GetAll(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.Transaction], error)
}

type gormTransactionRepository struct {
	db *gorm.DB
}

func NewTransactionRepository(db *gorm.DB) TransactionRepository {
	return &gormTransactionRepository{db: db}
}

func (r *gormTransactionRepository) WithTx(tx *gorm.DB) TransactionRepository {
	return &gormTransactionRepository{db: tx}
}

func (r *gormTransactionRepository) CreateTransaction(transaction *models.Transaction) error {
	return r.db.Create(transaction).Error
}

func (r *gormTransactionRepository) CreateRefTransaction(refTransaction *models.RefTransaction) error {
	return r.db.Create(refTransaction).Error
}

func (r *gormTransactionRepository) GetAll(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.Transaction], error) {
	db := r.db.Model(&models.Transaction{})
	db = ApplyFilters[models.Transaction](db, filters)

	paginatedResult, err := ApplyPagination[models.Transaction](db, page)
	if err != nil {
		return nil, err
	}

	return paginatedResult, nil
}
