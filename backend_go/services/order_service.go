package services

import (
	"frbktg/backend_go/apperrors"
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
	RenewSubscription(subscriptionID uint) error
}

type BuyResponse struct {
	Order        models.OrderSlimResponse `json:"order"`
	ProductName  string                   `json:"product_name"`
	ProductPrice float64                  `json:"product_price"`
	Balance      float64                  `json:"balance"`
}

type orderService struct {
	db                   *gorm.DB // For transaction management
	orderRepo            repositories.OrderRepository
	productRepo          repositories.ProductRepository
	botUserRepo          repositories.BotUserRepository
	transactionRepo      repositories.TransactionRepository
	userSubscriptionRepo repositories.UserSubscriptionRepository
	referralService      ReferralService
}

func NewOrderService(db *gorm.DB, orderRepo repositories.OrderRepository, productRepo repositories.ProductRepository, botUserRepo repositories.BotUserRepository, transactionRepo repositories.TransactionRepository, userSubscriptionRepo repositories.UserSubscriptionRepository, referralService ReferralService) OrderService {
	return &orderService{
		db:                   db,
		orderRepo:            orderRepo,
		productRepo:          productRepo,
		botUserRepo:          botUserRepo,
		transactionRepo:      transactionRepo,
		userSubscriptionRepo: userSubscriptionRepo,
		referralService:      referralService,
	}
}

func (s *orderService) GetOrders() ([]models.OrderResponse, error) {
	return s.orderRepo.GetOrders()
}

func (s *orderService) BuyFromBalance(userID int64, productID uint, quantity int, referralBotToken *string) (*BuyResponse, error) {
	var response *BuyResponse

	err := s.db.Transaction(func(tx *gorm.DB) error {
		user, err := s.botUserRepo.WithTx(tx).FindByTelegramID(userID)
		if err != nil {
			return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "BotUser", ID: uint(userID)}
		}

		product, err := s.productRepo.WithTx(tx).GetProductByID(productID)
		if err != nil {
			return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "Product", ID: productID}
		}

		if product.Type == "item" {
			stock, err := s.productRepo.WithTx(tx).GetStockForProduct(product.ID)
			if err != nil {
				return apperrors.New(500, "Failed to get stock for product", err)
			}
			if (stock - quantity) < 0 {
				return &apperrors.ErrOutOfStock{Base: apperrors.New(400, "", nil), ProductName: product.Name}
			}
		} else if product.Type == "subscription" {
			if quantity != 1 {
				return &apperrors.ErrValidation{Message: "quantity for subscription must be 1"}
			}
		}

		balance, err := s.botUserRepo.WithTx(tx).GetUserBalance(user.ID)
		if err != nil {
			return apperrors.New(500, "Failed to get user balance", err)
		}

		orderAmount := product.Price * float64(quantity)
		if balance < orderAmount {
			return apperrors.ErrInsufficientBalance
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
			return apperrors.New(500, "Failed to create order", err)
		}

		if product.Type == "subscription" {
			if err := s.handleSubscriptionPurchase(tx, user.ID, product, order.ID); err != nil {
				return err
			}
		}

		if err := s.createOrderTransactionsAndMovements(tx, user, product, *order, orderAmount, referralBotToken); err != nil {
			return err
		}

		newBalance := balance - orderAmount
		response = &BuyResponse{
			Order:        models.OrderSlimResponse(*order),
			ProductName:  product.Name,
			ProductPrice: product.Price,
			Balance:      newBalance,
		}

		return nil
	})

	return response, err
}

func (s *orderService) handleSubscriptionPurchase(tx *gorm.DB, userID uint, product *models.Product, orderID uint) error {
	subscriptionRepo := s.userSubscriptionRepo.WithTx(tx)

	existingSub, err := subscriptionRepo.FindActiveSubscription(userID, product.ID)
	if err != nil {
		return apperrors.New(500, "failed to find active subscription", err)
	}

	if existingSub != nil {
		// Extend existing subscription
		existingSub.ExpiresAt = existingSub.ExpiresAt.AddDate(0, 0, product.SubscriptionPeriodDays)
		existingSub.OrderID = orderID // Link to the new order
		if err := subscriptionRepo.UpdateSubscription(existingSub); err != nil {
			return apperrors.New(500, "failed to update subscription", err)
		}
	} else {
		// Create new subscription
		newSub := &models.UserSubscription{
			UserID:    userID,
			ProductID: product.ID,
			OrderID:   orderID,
			ExpiresAt: time.Now().AddDate(0, 0, product.SubscriptionPeriodDays),
			IsActive:  true,
		}
		if err := subscriptionRepo.CreateSubscription(newSub); err != nil {
			return apperrors.New(500, "failed to create subscription", err)
		}
	}

	return nil
}

func (s *orderService) CancelOrder(orderID uint) error {
	return s.db.Transaction(func(tx *gorm.DB) error {
		order, err := s.orderRepo.WithTx(tx).GetOrderForUpdate(orderID)
		if err != nil {
			return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "Order", ID: orderID}
		}

		if order.Status == "cancelled" {
			return &apperrors.ErrValidation{Base: apperrors.New(400, "", nil), Message: "order is already cancelled"}
		}

		product, err := s.productRepo.WithTx(tx).GetProductByID(order.ProductID)
		if err != nil {
			return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "Product", ID: order.ProductID}
		}

		if product.Type == "subscription" {
			return &apperrors.ErrValidation{Message: "cannot cancel a subscription order"}
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
			return apperrors.New(500, "Failed to create stock movement", err)
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
			return apperrors.New(500, "Failed to create transaction", err)
		}

		order.Status = "cancelled"
		if err := s.orderRepo.WithTx(tx).UpdateOrder(order); err != nil {
			return apperrors.New(500, "Failed to update order", err)
		}

		return nil
	})
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

	if product.Type == "item" {
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
	}

	if err := s.referralService.ProcessReferral(tx, referralBotToken, order, orderAmount); err != nil {
		return err
	}

	return nil
}

func (s *orderService) RenewSubscription(subscriptionID uint) error {
	return s.db.Transaction(func(tx *gorm.DB) error {
		subscription, err := s.userSubscriptionRepo.WithTx(tx).FindActiveSubscriptionByID(subscriptionID) // Assumes this method exists
		if err != nil {
			return apperrors.New(500, "failed to find subscription to renew", err)
		}
		if subscription == nil {
			return nil // Or log that subscription not found/already inactive
		}

		user, err := s.botUserRepo.WithTx(tx).FindByID(subscription.UserID)
		if err != nil {
			return &apperrors.ErrNotFound{Resource: "BotUser", ID: subscription.UserID}
		}

		product, err := s.productRepo.WithTx(tx).GetProductByID(subscription.ProductID)
		if err != nil {
			return &apperrors.ErrNotFound{Resource: "Product", ID: subscription.ProductID}
		}

		balance, err := s.botUserRepo.WithTx(tx).GetUserBalance(user.ID)
		if err != nil {
			return apperrors.New(500, "failed to get user balance for renewal", err)
		}

		if balance < product.Price {
			// Insufficient balance, deactivate subscription
			subscription.IsActive = false
			return s.userSubscriptionRepo.WithTx(tx).UpdateSubscription(subscription)
		}

		// Sufficient balance, proceed with renewal
		order := &models.Order{
			UserID:    user.ID,
			ProductID: product.ID,
			Quantity:  1,
			Amount:    product.Price,
			Status:    "success",
			CreatedAt: time.Now().UTC(),
		}

		if err := s.orderRepo.WithTx(tx).CreateOrder(order); err != nil {
			return apperrors.New(500, "failed to create renewal order", err)
		}

		// Create transaction for the renewal
		purchaseTransaction := &models.Transaction{
			UserID:      user.ID,
			OrderID:     &order.ID,
			Type:        models.Purchase,
			Amount:      -product.Price,
			Description: "Subscription renewal for " + product.Name,
			CreatedAt:   time.Now().UTC(),
		}
		if err := s.transactionRepo.WithTx(tx).CreateTransaction(purchaseTransaction); err != nil {
			return err
		}

		// Extend the subscription
		subscription.ExpiresAt = subscription.ExpiresAt.AddDate(0, 0, product.SubscriptionPeriodDays)
		subscription.OrderID = order.ID
		return s.userSubscriptionRepo.WithTx(tx).UpdateSubscription(subscription)
	})
}