package services

import (
	"fmt"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/external_providers"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"time"

	"gorm.io/gorm"
)

const percentDenominator = 100

// BuyRequest defines the parameters for buying a product.
type BuyRequest struct {
	UserID           int64
	ProductID        *uint
	Provider         *string
	ExternalProductID *string
	Quantity         int
	ReferralBotToken *string
}

type OrderService interface {
	GetOrders() ([]models.OrderResponse, error)
	BuyFromBalance(req BuyRequest) (*BuyResponse, error)
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
	db                   *gorm.DB
	orderRepo            repositories.OrderRepository
	productRepo          repositories.ProductRepository
	botUserRepo          repositories.BotUserRepository
	transactionRepo      repositories.TransactionRepository
	userSubscriptionRepo repositories.UserSubscriptionRepository
	referralService      ReferralService
	providerRegistry     *external_providers.ProviderRegistry
}

func NewOrderService(db *gorm.DB, orderRepo repositories.OrderRepository, productRepo repositories.ProductRepository, botUserRepo repositories.BotUserRepository, transactionRepo repositories.TransactionRepository, userSubscriptionRepo repositories.UserSubscriptionRepository, referralService ReferralService, providerRegistry *external_providers.ProviderRegistry) OrderService {
	return &orderService{
		db:                   db,
		orderRepo:            orderRepo,
		productRepo:          productRepo,
		botUserRepo:          botUserRepo,
		transactionRepo:      transactionRepo,
		userSubscriptionRepo: userSubscriptionRepo,
		referralService:      referralService,
		providerRegistry:     providerRegistry,
	}
}

func (s *orderService) GetOrders() ([]models.OrderResponse, error) {
	return s.orderRepo.GetOrders()
}

func (s *orderService) BuyFromBalance(req BuyRequest) (*BuyResponse, error) {
	var response *BuyResponse

	err := s.db.Transaction(func(tx *gorm.DB) error {
		user, err := s.botUserRepo.WithTx(tx).FindByTelegramID(req.UserID)
		if err != nil {
			return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "BotUser", ID: uint(req.UserID)}
		}

		var errPurchase error
		if req.ProductID != nil {
			response, errPurchase = s.handleInternalProductPurchase(tx, user, req)
		} else {
			response, errPurchase = s.handleExternalProductPurchase(tx, user, req)
		}
		return errPurchase
	})

	return response, err
}

func (s *orderService) handleInternalProductPurchase(tx *gorm.DB, user *models.BotUser, req BuyRequest) (*BuyResponse, error) {
	product, err := s.productRepo.WithTx(tx).GetProductByID(*req.ProductID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "Product", ID: *req.ProductID}
	}

	if product.Type == "item" {
		stock, err := s.productRepo.WithTx(tx).GetStockForProduct(product.ID)
		if err != nil {
			return nil, apperrors.New(500, "Failed to get stock for product", err)
		}
		if (stock - req.Quantity) < 0 {
			return nil, &apperrors.ErrOutOfStock{Base: apperrors.New(400, "", nil), ProductName: product.Name}
		}
	} else if product.Type == "subscription" {
		if req.Quantity != 1 {
			return nil, &apperrors.ErrValidation{Message: "quantity for subscription must be 1"}
		}
	}

	balance, err := s.botUserRepo.WithTx(tx).GetUserBalance(user.ID)
	if err != nil {
		return nil, apperrors.New(500, "Failed to get user balance", err)
	}

	orderAmount := product.Price * float64(req.Quantity)
	if balance < orderAmount {
		return nil, apperrors.ErrInsufficientBalance
	}

	order := &models.Order{
		UserID:    user.ID,
		ProductID: product.ID,
		Quantity:  req.Quantity,
		Amount:    orderAmount,
		Status:    "success",
		CreatedAt: time.Now().UTC(),
	}

	if err := s.orderRepo.WithTx(tx).CreateOrder(order); err != nil {
		return nil, apperrors.New(500, "Failed to create order", err)
	}

	if product.Type == "subscription" {
		if err := s.handleSubscriptionPurchase(tx, user.ID, product, order.ID, ""); err != nil {
			return nil, err
		}
	}

	if err := s.createOrderTransactionsAndMovements(tx, user, product, *order, orderAmount, req.ReferralBotToken); err != nil {
		return nil, err
	}

	newBalance := balance - orderAmount
	return &BuyResponse{
		Order:        models.OrderSlimResponse(*order),
		ProductName:  product.Name,
		ProductPrice: product.Price,
		Balance:      newBalance,
	}, nil
}

func (s *orderService) handleExternalProductPurchase(tx *gorm.DB, user *models.BotUser, req BuyRequest) (*BuyResponse, error) {
	provider, err := s.providerRegistry.GetProvider(*req.Provider)
	if err != nil {
		return nil, &apperrors.ErrValidation{Message: err.Error()}
	}

	// Get product details from provider to verify and get price
	extProducts, err := provider.GetProducts()
	if err != nil {
		return nil, apperrors.New(500, fmt.Sprintf("failed to get products from provider %s", *req.Provider), err)
	}

	var product *external_providers.ProviderProduct
	for _, p := range extProducts {
		if p.ExternalID == *req.ExternalProductID {
			product = &p
			break
		}
	}

	if product == nil {
		return nil, &apperrors.ErrNotFound{Resource: "External Product", ID: 0} // ID is not applicable here
	}

	// Check provider type and provision accordingly
	var provisionedID string
	if subProvider, ok := provider.(external_providers.SubscriptionProvider); ok {
		// It's a subscription provider
		provisionResult, err := subProvider.ProvisionSubscription(*req.ExternalProductID, *user, 30*24*time.Hour) // Assuming 30 days
		if err != nil {
			return nil, apperrors.New(500, "failed to provision external subscription", err)
		}
		provisionedID = provisionResult.ProvisionedID
	} else {
		// Here we would handle other provider types like ItemProvider
		return nil, apperrors.New(501, fmt.Sprintf("provider %s does not support the required provisioning interface", *req.Provider), nil)
	}

	balance, err := s.botUserRepo.WithTx(tx).GetUserBalance(user.ID)
	if err != nil {
		return nil, apperrors.New(500, "Failed to get user balance", err)
	}

	orderAmount := product.Price * float64(req.Quantity)
	if balance < orderAmount {
		return nil, apperrors.ErrInsufficientBalance
	}

	// Create a local product placeholder for the external product
	placeholderProduct := &models.Product{
		Name:       fmt.Sprintf("%s (%s)", product.Name, *req.Provider),
		Price:      product.Price,
		Type:       "subscription",
		CategoryID: 1, // Or a dedicated category for external products
	}
	if err := s.productRepo.WithTx(tx).CreateProduct(placeholderProduct); err != nil {
		return nil, apperrors.New(500, "failed to create placeholder product", err)
	}

	order := &models.Order{
		UserID:    user.ID,
		ProductID: placeholderProduct.ID, // Link to the placeholder
		Quantity:  req.Quantity,
		Amount:    orderAmount,
		Status:    "success",
		CreatedAt: time.Now().UTC(),
	}

	if err := s.orderRepo.WithTx(tx).CreateOrder(order); err != nil {
		return nil, apperrors.New(500, "Failed to create order for external product", err)
	}

	if err := s.handleSubscriptionPurchase(tx, user.ID, placeholderProduct, order.ID, provisionedID); err != nil {
		return nil, err
	}

	if err := s.createOrderTransactionsAndMovements(tx, user, placeholderProduct, *order, orderAmount, req.ReferralBotToken); err != nil {
		return nil, err
	}

	newBalance := balance - orderAmount
	return &BuyResponse{
		Order:        models.OrderSlimResponse(*order),
		ProductName:  product.Name,
		ProductPrice: product.Price,
		Balance:      newBalance,
	}, nil
}

func (s *orderService) handleSubscriptionPurchase(tx *gorm.DB, botUserID uint, product *models.Product, orderID uint, provisionedID string) error {
	subscriptionRepo := s.userSubscriptionRepo.WithTx(tx)

	existingSub, err := subscriptionRepo.FindActiveSubscription(botUserID, product.ID)
	if err != nil {
		return apperrors.New(500, "failed to find active subscription", err)
	}

	if existingSub != nil {
		existingSub.ExpiresAt = existingSub.ExpiresAt.AddDate(0, 0, product.SubscriptionPeriodDays)
		existingSub.OrderID = orderID
		if provisionedID != "" {
			existingSub.ProvisionedID = provisionedID
		}
		if err := subscriptionRepo.UpdateSubscription(existingSub); err != nil {
			return apperrors.New(500, "failed to update subscription", err)
		}
	} else {
		newSub := &models.UserSubscription{
			BotUserID:     botUserID,
			ProductID:     product.ID,
			OrderID:       orderID,
			ExpiresAt:     time.Now().AddDate(0, 0, product.SubscriptionPeriodDays),
			IsActive:      true,
			ProvisionedID: provisionedID,
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
			return &apperrors.ErrValidation{Message: "order is already cancelled"}
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
			Description: "Return for cancelled order",
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
			Description: "Refund for cancelled order",
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

func (s *orderService) RenewSubscription(subscriptionID uint) error {
	return s.db.Transaction(func(tx *gorm.DB) error {
		subscription, err := s.userSubscriptionRepo.WithTx(tx).FindActiveSubscriptionByID(subscriptionID)
		if err != nil {
			return apperrors.New(500, "failed to find subscription to renew", err)
		}
		if subscription == nil {
			return nil
		}

		user, err := s.botUserRepo.WithTx(tx).FindByID(subscription.BotUserID)
		if err != nil {
			return &apperrors.ErrNotFound{Resource: "BotUser", ID: subscription.BotUserID}
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
			subscription.IsActive = false
			return s.userSubscriptionRepo.WithTx(tx).UpdateSubscription(subscription)
		}

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

		subscription.ExpiresAt = subscription.ExpiresAt.AddDate(0, 0, product.SubscriptionPeriodDays)
		subscription.OrderID = order.ID
		return s.userSubscriptionRepo.WithTx(tx).UpdateSubscription(subscription)
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
			Description: "Sale to user",
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