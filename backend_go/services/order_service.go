package services

import (
	"encoding/json"
	"fmt"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/external_providers"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"log/slog"
	"strconv"
	"strings"
	"time"

	"gorm.io/gorm"
)

const percentDenominator = 100

// BuyRequest defines the parameters for buying a product.
type BuyRequest struct {
	UserID    int64 `json:"user_id"`
	ProductID uint  `json:"product_id"`
	Quantity  int   `json:"quantity"`
	BotID     uint  `json:"bot_id"`
}

type OrderService interface {
	GetOrders(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.OrderResponse], error)
	GetOrder(orderID uint) (*models.OrderResponse, error)
	BuyFromBalance(req BuyRequest) (*BuyResponse, error)
	CancelOrder(orderID uint) error
	RenewSubscription(subscriptionID uint) error
}

type BuyResponse struct {
	Order             models.OrderSlimResponse `json:"order"`
	ProductName       string                   `json:"product_name"`
	ProductPrice      float64                  `json:"product_price"`
	Balance           float64                  `json:"balance"`
	FulfilledContent  string                   `json:"fulfilled_content,omitempty"`
	ImageURL          string                   `json:"image_url,omitempty"`
	FulfilledImageURL string                   `json:"fulfilled_image_url,omitempty"`
}

type orderService struct {
	db                   *gorm.DB
	orderRepo            repositories.OrderRepository
	productRepo          repositories.ProductRepository
	botUserRepo          repositories.BotUserRepository
	transactionRepo      repositories.TransactionRepository
	userSubscriptionRepo repositories.UserSubscriptionRepository
	categoryRepo         repositories.CategoryRepository
	referralService      ReferralService
	botService           BotService
	productService       ProductService
	providerRegistry     *external_providers.ProviderRegistry
	webhookService       WebhookService
}

func NewOrderService(db *gorm.DB, orderRepo repositories.OrderRepository, productRepo repositories.ProductRepository, productService ProductService, botUserRepo repositories.BotUserRepository, transactionRepo repositories.TransactionRepository, userSubscriptionRepo repositories.UserSubscriptionRepository, categoryRepo repositories.CategoryRepository, referralService ReferralService, botService BotService, providerRegistry *external_providers.ProviderRegistry, webhookService WebhookService) OrderService {
	return &orderService{
		db:                   db,
		orderRepo:            orderRepo,
		productRepo:          productRepo,
		botUserRepo:          botUserRepo,
		transactionRepo:      transactionRepo,
		userSubscriptionRepo: userSubscriptionRepo,
		categoryRepo:         categoryRepo,
		referralService:      referralService,
		botService:           botService,
		productService:       productService,
		providerRegistry:     providerRegistry,
		webhookService:       webhookService,
	}
}

func (s *orderService) GetOrders(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.OrderResponse], error) {
	return s.orderRepo.GetOrders(page, filters)
}

func (s *orderService) GetOrder(orderID uint) (*models.OrderResponse, error) {
	order, err := s.orderRepo.GetOrderByID(orderID)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Resource: "Order", ID: orderID}
	}

	imageURL := ""
	if order.Product.ImageID != nil {
		imageURL = fmt.Sprintf("/images/%s", order.Product.ImageID.String())
	}

	fulfilledImageURL := ""
	if order.FulfilledImageID != nil {
		fulfilledImageURL = fmt.Sprintf("/images/%s", order.FulfilledImageID.String())
	}

	return &models.OrderResponse{
		ID:                order.ID,
		UserID:            order.UserID,
		ProductID:         order.ProductID,
		Quantity:          order.Quantity,
		Amount:            order.Amount,
		Status:            order.Status,
		CreatedAt:         order.CreatedAt,
		UserTelegramID:    0, // TODO: Populate this field
		ProductName:       order.Product.Name,
		FulfilledContent:  order.FulfilledContent,
		FulfilledImageURL: fulfilledImageURL,
		ImageURL:          imageURL,
	}, nil
}

func (s *orderService) BuyFromBalance(req BuyRequest) (*BuyResponse, error) {
	var response *BuyResponse

	err := s.db.Transaction(func(tx *gorm.DB) error {
		user, err := s.botUserRepo.WithTx(tx).FindByTelegramID(req.UserID)
		if err != nil {
			return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "BotUser", ID: uint(req.UserID)}
		}

		product, err := s.productRepo.WithTx(tx).GetProductByID(req.ProductID)
		if err != nil {
			return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "Product", ID: req.ProductID}
		}

		if product.Type == "item" {
			stock, err := s.productRepo.WithTx(tx).GetStockForProduct(product.ID)
			if err != nil {
				return apperrors.New(500, "Failed to get stock for product", err)
			}
			if (stock - req.Quantity) < 0 {
				return &apperrors.ErrOutOfStock{Base: apperrors.New(400, "", nil), ProductName: product.Name}
			}
		} else if product.Type == "subscription" {
			if req.Quantity != 1 {
				return &apperrors.ErrValidation{Message: "quantity for subscription must be 1"}
			}
		}

		productPrice, err := s.productService.CalculateProductPrice(product, "")
		if err != nil {
			return err
		}

		balance, err := s.botUserRepo.WithTx(tx).GetUserBalance(user.ID)
		if err != nil {
			return apperrors.New(500, "Failed to get user balance", err)
		}

		orderAmount := productPrice * float64(req.Quantity)
		if balance < orderAmount {
			return apperrors.ErrInsufficientBalance
		}

		order := &models.Order{
			UserID:    user.ID,
			ProductID: product.ID,
			Quantity:  req.Quantity,
			Amount:    orderAmount,
			Status:    "success",
			BotID:     req.BotID,
			CreatedAt: time.Now().UTC(),
		}

		if err := s.orderRepo.WithTx(tx).CreateOrder(order); err != nil {
			return apperrors.New(500, "Failed to create order", err)
		}

		if product.Type == "subscription" {
			var provisionedID string
			var provisionResult *external_providers.ProvisioningResult

			if product.ProviderName != nil && *product.ProviderName != "" {
				provider, err := s.providerRegistry.GetProvider(*product.ProviderName)
				if err != nil {
					return &apperrors.ErrValidation{Message: err.Error()}
				}

				if subProvider, ok := provider.(external_providers.SubscriptionProvider); ok {
					provisionResult, err = subProvider.ProvisionSubscription(*product.ExternalID, *user, 30*24*time.Hour) // Assuming 30 days
					if err != nil {
						return apperrors.New(500, "failed to provision external subscription", err)
					}
					provisionedID = provisionResult.ProvisionedID
				} else {
					return apperrors.New(501, fmt.Sprintf("provider %s does not support the required provisioning interface", *product.ProviderName), nil)
				}
			}

			if err := s.handleSubscriptionPurchase(tx, user.ID, product, order.ID, provisionedID, provisionResult.Details); err != nil {
				return err
			}

			// Augment fulfillment content with dynamic data for the response
			dynamicContent := ""
			if username, ok := provisionResult.Details["username"].(string); ok {
				dynamicContent += fmt.Sprintf("\nUsername: %s", username)
			}
			if password, ok := provisionResult.Details["password"].(string); ok {
				dynamicContent += fmt.Sprintf("\nPassword: %s", password)
			}
			if product.FulfillmentText.Valid {
				order.FulfilledContent = product.FulfillmentText.String + dynamicContent
			}

		}

		if err := s.createOrderTransactionsAndMovements(tx, user, product, *order, orderAmount); err != nil {
			return err
		}

		// --- Fulfillment Logic ---
		// For non-subscription items
		if product.Type != "subscription" {
			if product.FulfillmentText.Valid {
				order.FulfilledContent = product.FulfillmentText.String
			}
			if product.FulfillmentImageID != nil {
				order.FulfilledImageID = product.FulfillmentImageID
			}
		}

		if (order.FulfilledContent != "") || (order.FulfilledImageID != nil) {
			if err := s.orderRepo.WithTx(tx).UpdateOrder(order); err != nil {
				// Log the error but don't fail the transaction
				slog.Error("failed to update order with fulfilled content", "order_id", order.ID, "error", err)
			}
		}
		// --- End Fulfillment Logic ---

		newBalance := balance - orderAmount
		imageURL := ""
		if product.ImageID != nil {
			imageURL = fmt.Sprintf("/images/%s", product.ImageID.String())
		}

		fulfilledImageURL := ""
		if order.FulfilledImageID != nil {
			fulfilledImageURL = fmt.Sprintf("/images/%s", order.FulfilledImageID.String())
		}

		response = &BuyResponse{
			Order: models.OrderSlimResponse{
				ID:        order.ID,
				UserID:    order.UserID,
				ProductID: order.ProductID,
				Quantity:  order.Quantity,
				Amount:    order.Amount,
				Status:    order.Status,
				CreatedAt: order.CreatedAt,
			},
			ProductName:       product.Name,
			ProductPrice:      productPrice,
			Balance:           newBalance,
			FulfilledContent:  order.FulfilledContent,
			ImageURL:          imageURL,
			FulfilledImageURL: fulfilledImageURL,
		}
		return nil
	})

	return response, err
}

func (s *orderService) handleSubscriptionPurchase(tx *gorm.DB, botUserID uint, product *models.Product, orderID uint, provisionedID string, details map[string]interface{}) error {
	subscriptionRepo := s.userSubscriptionRepo.WithTx(tx)

	if details == nil {
		details = make(map[string]interface{})
	}

	// Parse host and port from fulfillment content and add to details
	if product.FulfillmentText.Valid {
		lines := strings.Split(product.FulfillmentText.String, "\n")
		for _, line := range lines {
			parts := strings.SplitN(line, ":", 2)
			if len(parts) == 2 {
				key := strings.TrimSpace(strings.ToLower(parts[0]))
				value := strings.TrimSpace(parts[1])
				if key == "host" {
					details["host"] = value
				} else if key == "port" {
					if port, err := strconv.Atoi(value); err == nil {
						details["port"] = port
					}
				}
			}
		}
	}
	// Convert details to JSON
	detailsJSON, err := json.Marshal(details)
	if err != nil {
		return apperrors.New(500, "failed to marshal subscription details", err)
	}

	var expiresAt time.Time
	if expires, ok := details["expires"].(time.Time); ok {
		expiresAt = expires
	} else {
		expiresAt = time.Now().AddDate(0, 0, product.SubscriptionPeriodDays)
	}

	existingSub, err := subscriptionRepo.FindActiveSubscription(botUserID, product.ID)
	if err != nil {
		return apperrors.New(500, "failed to find active subscription", err)
	}

	if existingSub != nil {
		// Extend existing subscription
		existingSub.ExpiresAt = existingSub.ExpiresAt.AddDate(0, 0, product.SubscriptionPeriodDays)
		existingSub.OrderID = orderID // Link to the new order
		if provisionedID != "" {
			existingSub.ProvisionedID = provisionedID
		}
		existingSub.Details = detailsJSON
		if err := subscriptionRepo.UpdateSubscription(existingSub); err != nil {
			return apperrors.New(500, "failed to update subscription", err)
		}
	} else {
		// Create new subscription
		newSub := &models.UserSubscription{
			BotUserID:     botUserID,
			ProductID:     product.ID,
			OrderID:       orderID,
			ExpiresAt:     expiresAt,
			IsActive:      true,
			ProvisionedID: provisionedID,
			Details:       detailsJSON,
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

		if err := s.botUserRepo.WithTx(tx).UpdateBalance(order.UserID, order.Amount); err != nil {
			return apperrors.New(500, "Failed to update user balance", err)
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

		if err := s.botUserRepo.WithTx(tx).UpdateBalance(user.ID, -product.Price); err != nil {
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

	if err := s.botUserRepo.WithTx(tx).UpdateBalance(user.ID, -orderAmount); err != nil {
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

	if err := s.referralService.ProcessReferral(tx, order.BotID, order, orderAmount); err != nil {
		return err
	}

	return nil
}
