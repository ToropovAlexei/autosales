package mocks

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"

	"github.com/stretchr/testify/mock"
	"gorm.io/gorm"
)

type MockTransactionRepository struct{ mock.Mock }

func (m *MockTransactionRepository) WithTx(tx *gorm.DB) repositories.TransactionRepository { m.Called(tx); return m }
func (m *MockTransactionRepository) CreateTransaction(transaction *models.Transaction) error {
	return m.Called(transaction).Error(0)
}
func (m *MockTransactionRepository) CreateRefTransaction(refTransaction *models.RefTransaction) error {
	return m.Called(refTransaction).Error(0)
}
func (m *MockTransactionRepository) GetAll(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.Transaction], error) {
	args := m.Called(page, filters)
	return args.Get(0).(*models.PaginatedResult[models.Transaction]), args.Error(1)
}