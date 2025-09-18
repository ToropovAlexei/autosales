package services

import (
	"errors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"strconv"
	"time"

	"gorm.io/gorm"
)

const percentDenominator = 100

type OrderService interface {
	GetOrders() ([]models.OrderResponse, error)
	BuyFromBalance(userID int64, productID uint, quantity int, referralBotToken *string) (*BuyResponse, error)
	CancelOrder(orderID uint) error
}

type BuyResponse struct {
	Order        models.OrderSlimResponse `json:"order"`
	ProductName  string                   `json:"product_name"`
	ProductPrice float64                  `json:"product_price"`
	Balance      float64                  `json:"balance"`
}

type orderService struct {
	db                    *gorm.DB // For transaction management
	orderRepo             repositories.OrderRepository
	productRepo           repositories.ProductRepository
	botUserRepo           repositories.BotUserRepository
	transactionRepo       repositories.TransactionRepository
	// TODO: Add referral repositories when they are refactored
}

func NewOrderService(db *gorm.DB, orderRepo repositories.OrderRepository, productRepo repositories.ProductRepository, botUserRepo repositories.BotUserRepository, transactionRepo repositories.TransactionRepository) OrderService {
	return &orderService{
		db:                    db,
		orderRepo:             orderRepo,
		productRepo:           productRepo,
		botUserRepo:           botUserRepo,
		transactionRepo:       transactionRepo,
	}
}

func (s *orderService) GetOrders() ([]models.OrderResponse, error) {
	return s.orderRepo.GetOrders()
}

func (s *orderService) BuyFromBalance(userID int64, productID uint, quantity int, referralBotToken *string) (*BuyResponse, error) {
	// 1. Validate data
	user, err := s.botUserRepo.FindByTelegramID(userID)
	if err != nil {
		return nil, errors.New("bot user not found")
	}

	product, err := s.productRepo.GetProductByID(productID)
	if err != nil {
		return nil, errors.New("product not found")
	}

	stock, err := s.productRepo.GetStockForProduct(product.ID)
	if err != nil {
		return nil, err
	}
	if (stock - quantity) < 0 {
		return nil, errors.New("product out of stock")
	}

	balance, err := s.botUserRepo.GetUserBalance(user.ID)
	if err != nil {
		return nil, err
	}

	orderAmount := product.Price * float64(quantity)
	if balance < orderAmount {
		return nil, errors.New("insufficient balance")
	}

	// 2. Perform operations in a transaction
	tx := s.db.Begin()
	if tx.Error != nil {
		return nil, tx.Error
	}

	order := &models.Order{
		UserID:    user.ID,
		ProductID: product.ID,
		Quantity:  quantity,
		Amount:    orderAmount,
		Status:    "success",
		CreatedAt: time.Now().UTC(),
	}

	if err := s.orderRepo.WithTx(tx).CreateOrder(order); err != nil {
		tx.Rollback()
		return nil, err
	}

	if err := s.createOrderTransactionsAndMovements(tx, user, product, *order, orderAmount, referralBotToken); err != nil {
		tx.Rollback()
		return nil, err
	}

	if err := tx.Commit().Error; err != nil {
		return nil, err
	}

	// 3. Return response
	newBalance := balance - orderAmount
	response := &BuyResponse{
		Order:        models.OrderSlimResponse(*order),
		ProductName:  product.Name,
		ProductPrice: product.Price,
		Balance:      newBalance,
	}

	return response, nil
}

func (s *orderService) CancelOrder(orderID uint) error {
	tx := s.db.Begin()
	if tx.Error != nil {
		return tx.Error
	}

	order, err := s.orderRepo.WithTx(tx).GetOrderForUpdate(orderID)
	if err != nil {
		tx.Rollback()
		return errors.New("order not found")
	}

	if order.Status == "cancelled" {
		tx.Rollback()
		return errors.New("order is already cancelled")
	}

	returnMovement := &models.StockMovement{
		OrderID:     &order.ID,
		ProductID:   order.ProductID,
		Type:        models.Return,
		Quantity:    order.Quantity,
		Description: "Return for cancelled order " + strconv.FormatUint(uint64(order.ID), 10),
		CreatedAt:   time.Now().UTC(),
	}
	if err := s.productRepo.WithTx(tx).CreateStockMovement(returnMovement); err != nil {
		tx.Rollback()
		return err
	}

	refundTransaction := &models.Transaction{
		UserID:      order.UserID,
		OrderID:     &order.ID,
		Type:        models.Deposit,
		Amount:      order.Amount,
		Description: "Refund for cancelled order " + strconv.FormatUint(uint64(order.ID), 10),
		CreatedAt:   time.Now().UTC(),
	}
	if err := s.transactionRepo.WithTx(tx).CreateTransaction(refundTransaction); err != nil {
		tx.Rollback()
		return err
	}

	order.Status = "cancelled"
	if err := s.orderRepo.WithTx(tx).UpdateOrder(order); err != nil {
		tx.Rollback()
		return err
	}

	return tx.Commit().Error
}

func (s *orderService) createOrderTransactionsAndMovements(
	tx *gorm.DB,
	user *models.BotUser,
	product *models.Product,
	order models.Order,
	orderAmount float64,
	referralBotToken *string,
) error {
	purchaseTransaction := &models.Transaction{
		UserID:      user.ID,
		OrderID:     &order.ID,
		Type:        models.Purchase,
		Amount:      -orderAmount,
		Description: "Purchase of " + product.Name,
		CreatedAt:   time.Now().UTC(),
	}
	if err := s.transactionRepo.WithTx(tx).CreateTransaction(purchaseTransaction); err != nil {
		return err
	}

	saleMovement := &models.StockMovement{
		OrderID:     &order.ID,
		ProductID:   product.ID,
		Type:        models.Sale,
		Quantity:    -order.Quantity,
		Description: "Sale to user " + strconv.FormatUint(uint64(user.ID), 10),
		CreatedAt:   time.Now().UTC(),
	}
	if err := s.productRepo.WithTx(tx).CreateStockMovement(saleMovement); err != nil {
		return err
	}

	if referralBotToken != nil {
		// TODO: This part requires refactoring of referral-related tables and logic
		// For now, we keep the direct DB call, but it should be moved to a referral service/repository
		var refBot models.ReferralBot
		if err := tx.Where("bot_token = ?", *referralBotToken).First(&refBot).Error; err == nil && refBot.IsActive {
			var seller models.User
			if sellerErr := tx.First(&seller, refBot.SellerID).Error; sellerErr == nil &&
				seller.ReferralProgramEnabled && seller.ReferralPercentage > 0 {
				refShare := orderAmount * (seller.ReferralPercentage / percentDenominator)
				refTransaction := &models.RefTransaction{
					RefOwnerID: refBot.OwnerID,
					SellerID:   seller.ID,
					OrderID:    order.ID,
					Amount:     orderAmount,
					RefShare:   refShare,
					CreatedAt:  time.Now().UTC(),
				}
				if createErr := s.transactionRepo.WithTx(tx).CreateRefTransaction(refTransaction); createErr != nil {
					return createErr
				}
			}
		}
	}
	return nil
}
