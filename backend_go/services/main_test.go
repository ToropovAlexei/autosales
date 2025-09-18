package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/stretchr/testify/mock"
	"gorm.io/gorm"
)

// This file contains mock definitions used across multiple test files in this package.

// --- Mocks ---

type MockUserRepository struct{ mock.Mock }
func (m *MockUserRepository) WithTx(tx *gorm.DB) repositories.UserRepository { m.Called(tx); return m }
func (m *MockUserRepository) UpdateReferralSettings(user *models.User, enabled bool, percentage float64) error { return m.Called(user, enabled, percentage).Error(0) }
func (m *MockUserRepository) FindSellerSettings() (*models.User, error) { args := m.Called(); return args.Get(0).(*models.User), args.Error(1) }
func (m *MockUserRepository) FindByID(id uint) (*models.User, error) { args := m.Called(id); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.User), args.Error(1) }
func (m *MockUserRepository) FindByEmail(email string) (*models.User, error) { args := m.Called(email); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.User), args.Error(1) }

type MockBotUserRepository struct{ mock.Mock }
func (m *MockBotUserRepository) FindByTelegramID(telegramID int64) (*models.BotUser, error) { args := m.Called(telegramID); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.BotUser), args.Error(1) }
func (m *MockBotUserRepository) FindByID(id uint) (*models.BotUser, error) { args := m.Called(id); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.BotUser), args.Error(1) }
func (m *MockBotUserRepository) Create(user *models.BotUser) error { return m.Called(user).Error(0) }
func (m *MockBotUserRepository) Update(user *models.BotUser) error { return m.Called(user).Error(0) }
func (m *MockBotUserRepository) UpdateCaptchaStatus(user *models.BotUser, hasPassed bool) error { return m.Called(user, hasPassed).Error(0) }
func (m *MockBotUserRepository) GetUserBalance(userID uint) (float64, error) { args := m.Called(userID); return args.Get(0).(float64), args.Error(1) }
func (m *MockBotUserRepository) GetUserTransactions(userID uint) ([]models.Transaction, error) { args := m.Called(userID); return args.Get(0).([]models.Transaction), args.Error(1) }

type MockProductRepository struct{ mock.Mock }
func (m *MockProductRepository) WithTx(tx *gorm.DB) repositories.ProductRepository { m.Called(tx); return m }
func (m *MockProductRepository) GetProducts(categoryIDs []string) ([]models.Product, error) { return nil, nil }
func (m *MockProductRepository) GetProductByID(id uint) (*models.Product, error) { args := m.Called(id); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.Product), args.Error(1) }
func (m *MockProductRepository) CreateProduct(product *models.Product) error { return m.Called(product).Error(0) }
func (m *MockProductRepository) UpdateProduct(product *models.Product, data models.Product) error { return m.Called(product, data).Error(0) }
func (m *MockProductRepository) DeleteProduct(product *models.Product) error { return m.Called(product).Error(0) }
func (m *MockProductRepository) GetStockForProduct(productID uint) (int, error) { args := m.Called(productID); return args.Int(0), args.Error(1) }
func (m *MockProductRepository) CreateStockMovement(movement *models.StockMovement) error { return m.Called(movement).Error(0) }
func (m *MockProductRepository) FindCategoryByID(id uint) (*models.Category, error) { args := m.Called(id); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.Category), args.Error(1) }

type MockOrderRepository struct{ mock.Mock }
func (m *MockOrderRepository) WithTx(tx *gorm.DB) repositories.OrderRepository { m.Called(tx); return m }
func (m *MockOrderRepository) CreateOrder(order *models.Order) error { return m.Called(order).Error(0) }
func (m *MockOrderRepository) GetOrderForUpdate(id uint) (*models.Order, error) { args := m.Called(id); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.Order), args.Error(1) }
func (m *MockOrderRepository) UpdateOrder(order *models.Order) error { return m.Called(order).Error(0) }
func (m *MockOrderRepository) GetOrders() ([]models.OrderResponse, error) { args := m.Called(); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).([]models.OrderResponse), args.Error(1) }

type MockTransactionRepository struct{ mock.Mock }
func (m *MockTransactionRepository) WithTx(tx *gorm.DB) repositories.TransactionRepository { m.Called(tx); return m }
func (m *MockTransactionRepository) CreateTransaction(transaction *models.Transaction) error { return m.Called(transaction).Error(0) }
func (m *MockTransactionRepository) CreateRefTransaction(refTransaction *models.RefTransaction) error { return m.Called(refTransaction).Error(0) }
func (m *MockTransactionRepository) GetAll() ([]models.Transaction, error) { return nil, nil }

type MockReferralService struct{ mock.Mock }
func (m *MockReferralService) ProcessReferral(tx *gorm.DB, token *string, order models.Order, amount float64) error { return m.Called(tx, token, order, amount).Error(0) }
func (m *MockReferralService) CreateReferralBot(ownerTelegramID int64, sellerID uint, botToken string) (*models.ReferralBot, error) { return nil, nil }
func (m *MockReferralService) GetAllReferralBots() ([]models.ReferralBotResponse, error) { return nil, nil }
func (m *MockReferralService) GetAdminInfoForSeller(sellerID uint) ([]models.ReferralBotAdminInfo, error) { return nil, nil }
func (m *MockReferralService) ToggleReferralBotStatus(botID uint, sellerID uint) (*models.ReferralBot, error) { return nil, nil }

type MockTokenService struct{ mock.Mock }
func (m *MockTokenService) GenerateToken(user *models.User, secretKey string, expireMinutes int) (string, error) { args := m.Called(user, secretKey, expireMinutes); return args.String(0), args.Error(1) }
func (m *MockTokenService) ValidateToken(tokenString string, secretKey string) (*jwt.Token, error) { args := m.Called(tokenString, secretKey); return args.Get(0).(*jwt.Token), args.Error(1) }

type MockAdminRepository struct{ mock.Mock }
func (m *MockAdminRepository) GetActiveBotUsers() ([]models.BotUser, error) { args := m.Called(); return args.Get(0).([]models.BotUser), args.Error(1) }
func (m *MockAdminRepository) GetBotUserByID(id uint) (*models.BotUser, error) { args := m.Called(id); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.BotUser), args.Error(1) }
func (m *MockAdminRepository) SoftDeleteBotUser(user *models.BotUser) error { return m.Called(user).Error(0) }

type MockCategoryRepository struct{ mock.Mock }
func (m *MockCategoryRepository) GetAll() ([]models.Category, error) { args := m.Called(); return args.Get(0).([]models.Category), args.Error(1) }
func (m *MockCategoryRepository) GetByID(id uint) (*models.Category, error) { args := m.Called(id); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.Category), args.Error(1) }
func (m *MockCategoryRepository) Create(category *models.Category) error { return m.Called(category).Error(0) }
func (m *MockCategoryRepository) Update(category *models.Category, data models.Category) error { return m.Called(category, data).Error(0) }
func (m *MockCategoryRepository) Delete(category *models.Category) error { return m.Called(category).Error(0) }

type MockDashboardRepository struct{ mock.Mock }
func (m *MockDashboardRepository) CountTotalUsers() (int64, error) { args := m.Called(); return args.Get(0).(int64), args.Error(1) }
func (m *MockDashboardRepository) CountUsersWithPurchases() (int64, error) { args := m.Called(); return args.Get(0).(int64), args.Error(1) }
func (m *MockDashboardRepository) CountAvailableProducts() (int64, error) { args := m.Called(); return args.Get(0).(int64), args.Error(1) }
func (m *MockDashboardRepository) GetSalesCountForPeriod(start, end time.Time) (int64, error) { args := m.Called(start, end); return args.Get(0).(int64), args.Error(1) }
func (m *MockDashboardRepository) GetTotalRevenueForPeriod(start, end time.Time) (float64, error) { args := m.Called(start, end); return args.Get(0).(float64), args.Error(1) }