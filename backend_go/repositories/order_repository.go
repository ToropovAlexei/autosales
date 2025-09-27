package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

// OrderRepository handles database operations related to orders.
// It is designed to be used within a single database transaction.
type OrderRepository interface {
	// WithTx returns a new instance of the repository with the given transaction.
	WithTx(tx *gorm.DB) OrderRepository
	CreateOrder(order *models.Order) error
	GetOrderForUpdate(orderID uint) (*models.Order, error)
	UpdateOrder(order *models.Order) error
	GetOrders() ([]models.OrderResponse, error)
	FindOrdersByBotUserID(botUserID uint) ([]models.Order, error)
}

type gormOrderRepository struct {
	db *gorm.DB
}

func NewOrderRepository(db *gorm.DB) OrderRepository {
	return &gormOrderRepository{db: db}
}

func (r *gormOrderRepository) WithTx(tx *gorm.DB) OrderRepository {
	return &gormOrderRepository{db: tx}
}

func (r *gormOrderRepository) CreateOrder(order *models.Order) error {
	return r.db.Create(order).Error
}

func (r *gormOrderRepository) GetOrderForUpdate(orderID uint) (*models.Order, error) {
	var order models.Order
	if err := r.db.First(&order, orderID).Error; err != nil {
		return nil, err
	}
	return &order, nil
}

func (r *gormOrderRepository) UpdateOrder(order *models.Order) error {
	return r.db.Save(order).Error
}

func (r *gormOrderRepository) GetOrders() ([]models.OrderResponse, error) {
	var response []models.OrderResponse
	if err := r.db.Table("orders").
		Select("orders.*, bot_users.telegram_id as user_telegram_id, products.name as product_name").
		Joins("join bot_users on bot_users.id = orders.user_id").
		Joins("join products on products.id = orders.product_id").
		Order("orders.created_at desc").
		Scan(&response).Error; err != nil {
		return nil, err
	}
	return response, nil
}

func (r *gormOrderRepository) FindOrdersByBotUserID(botUserID uint) ([]models.Order, error) {
	var orders []models.Order
	if err := r.db.Preload("Product").Where("user_id = ?", botUserID).Order("created_at desc").Find(&orders).Error; err != nil {
		return nil, err
	}
	return orders, nil
}
