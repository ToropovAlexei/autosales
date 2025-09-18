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
	GetAll() ([]models.Transaction, error)
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

func (r *gormTransactionRepository) GetAll() ([]models.Transaction, error) {
	var transactions []models.Transaction
	if err := r.db.Order("created_at desc").Find(&transactions).Error; err != nil {
		return nil, err
	}
	return transactions, nil
}
