package services

import (
	"frbktg/backend_go/models"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

// --- Test Setup ---
func setupOrderServiceTest() (*orderService, *MockOrderRepository, *MockProductRepository, *MockBotUserRepository, *MockTransactionRepository, *MockReferralService) {
	db, _ := gorm.Open(sqlite.Open("file::memory:"), &gorm.Config{})
	orderRepo := new(MockOrderRepository)
	productRepo := new(MockProductRepository)
	botUserRepo := new(MockBotUserRepository)
	transactionRepo := new(MockTransactionRepository)
	referralService := new(MockReferralService)
	sut := NewOrderService(db, orderRepo, productRepo, botUserRepo, transactionRepo, referralService).(*orderService)
	return sut, orderRepo, productRepo, botUserRepo, transactionRepo, referralService
}

// --- Tests ---

func TestOrderService_BuyFromBalance_Success(t *testing.T) {
	sut, orderRepo, productRepo, botUserRepo, transactionRepo, referralService := setupOrderServiceTest()
	user, product, quantity, orderAmount := &models.BotUser{ID: 1, TelegramID: 123}, &models.Product{ID: 10, Name: "Item", Price: 50.0}, 2, 100.0

	botUserRepo.On("FindByTelegramID", user.TelegramID).Return(user, nil)
	productRepo.On("GetProductByID", product.ID).Return(product, nil)
	productRepo.On("GetStockForProduct", product.ID).Return(5, nil)
	botUserRepo.On("GetUserBalance", user.ID).Return(150.0, nil)

	orderRepo.On("WithTx", mock.Anything).Return(orderRepo)
	orderRepo.On("CreateOrder", mock.AnythingOfType("*models.Order")).Return(nil)
	transactionRepo.On("WithTx", mock.Anything).Return(transactionRepo)
	transactionRepo.On("CreateTransaction", mock.AnythingOfType("*models.Transaction")).Return(nil)
	productRepo.On("WithTx", mock.Anything).Return(productRepo)
	productRepo.On("CreateStockMovement", mock.AnythingOfType("*models.StockMovement")).Return(nil)
	referralService.On("ProcessReferral", mock.Anything, mock.Anything, mock.Anything, mock.Anything).Return(nil)

	resp, err := sut.BuyFromBalance(user.TelegramID, product.ID, quantity, nil)

	assert.NoError(t, err)
	assert.NotNil(t, resp)
	assert.Equal(t, product.Name, resp.ProductName)
	assert.Equal(t, 50.0, resp.Balance)
	assert.Equal(t, orderAmount, resp.Order.Amount)
	mock.AssertExpectationsForObjects(t, orderRepo, productRepo, botUserRepo, transactionRepo, referralService)
}

func TestOrderService_BuyFromBalance_OutOfStock(t *testing.T) {
	sut, _, productRepo, botUserRepo, _, _ := setupOrderServiceTest()
	user, product := &models.BotUser{ID: 1, TelegramID: 123}, &models.Product{ID: 10}

	botUserRepo.On("FindByTelegramID", user.TelegramID).Return(user, nil)
	productRepo.On("GetProductByID", product.ID).Return(product, nil)
	productRepo.On("GetStockForProduct", product.ID).Return(1, nil) // Not enough

	resp, err := sut.BuyFromBalance(user.TelegramID, product.ID, 2, nil)

	assert.Error(t, err)
	assert.Nil(t, resp)
	assert.Equal(t, "product out of stock", err.Error())
}

func TestOrderService_BuyFromBalance_InsufficientBalance(t *testing.T) {
	sut, _, productRepo, botUserRepo, _, _ := setupOrderServiceTest()
	user, product := &models.BotUser{ID: 1, TelegramID: 123}, &models.Product{ID: 10, Price: 50.0}

	botUserRepo.On("FindByTelegramID", user.TelegramID).Return(user, nil)
	productRepo.On("GetProductByID", product.ID).Return(product, nil)
	productRepo.On("GetStockForProduct", product.ID).Return(5, nil)
	botUserRepo.On("GetUserBalance", user.ID).Return(99.0, nil) // Not enough

	resp, err := sut.BuyFromBalance(user.TelegramID, product.ID, 2, nil)

	assert.Error(t, err)
	assert.Nil(t, resp)
	assert.Equal(t, "insufficient balance", err.Error())
}

func TestOrderService_CancelOrder_Success(t *testing.T) {
	sut, orderRepo, productRepo, _, transactionRepo, _ := setupOrderServiceTest()
	order := &models.Order{ID: 1, Status: "success", Quantity: 2, Amount: 100.0}

	orderRepo.On("WithTx", mock.Anything).Return(orderRepo)
	orderRepo.On("GetOrderForUpdate", order.ID).Return(order, nil)
	orderRepo.On("UpdateOrder", mock.AnythingOfType("*models.Order")).Return(nil)
	transactionRepo.On("WithTx", mock.Anything).Return(transactionRepo)
	transactionRepo.On("CreateTransaction", mock.AnythingOfType("*models.Transaction")).Return(nil)
	productRepo.On("WithTx", mock.Anything).Return(productRepo)
	productRepo.On("CreateStockMovement", mock.AnythingOfType("*models.StockMovement")).Return(nil)

	err := sut.CancelOrder(order.ID)

	assert.NoError(t, err)
	mock.AssertExpectationsForObjects(t, orderRepo, productRepo, transactionRepo)
}

func TestOrderService_CancelOrder_AlreadyCancelled(t *testing.T) {
	sut, orderRepo, _, _, _, _ := setupOrderServiceTest()
	order := &models.Order{ID: 1, Status: "cancelled"}

	orderRepo.On("WithTx", mock.Anything).Return(orderRepo)
	orderRepo.On("GetOrderForUpdate", order.ID).Return(order, nil)

	err := sut.CancelOrder(order.ID)

	assert.Error(t, err)
	assert.Equal(t, "order is already cancelled", err.Error())
}