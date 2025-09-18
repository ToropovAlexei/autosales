package routers

import (
	"errors"
	"net/http"
	"strconv"
	"time"

	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"
	"frbktg/backend_go/responses"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
)

func (r *Router) OrdersRouter(router *gin.Engine) {
	service := router.Group("/api/orders")
	service.Use(middleware.ServiceTokenMiddleware(r.appSettings))
	{
		service.POST("/buy-from-balance", r.buyFromBalanceHandler)
	}

	auth := router.Group("/api/orders")
	auth.Use(middleware.AuthMiddleware(r.appSettings, r.db))
	{
		auth.GET("", r.getOrdersHandler)
		auth.POST("/:id/cancel", r.cancelOrderHandler)
	}
}

type OrderCreate struct {
	UserID           int64   `json:"user_id"`
	ProductID        uint    `json:"product_id"`
	Quantity         int     `json:"quantity"`
	ReferralBotToken *string `json:"referral_bot_token"`
}

type BuyResponse struct {
	Order        models.OrderSlimResponse `json:"order"`
	ProductName  string                   `json:"product_name"`
	ProductPrice float64                  `json:"product_price"`
	Balance      float64                  `json:"balance"`
}

type OrderValidationData struct {
	JSON        OrderCreate
	User        models.BotUser
	Product     models.Product
	Balance     float64
	OrderAmount float64
}

func (r *Router) buyFromBalanceHandler(c *gin.Context) {
	data, err := r.validateAndRetrieveOrderData(c)
	if err != nil {
		return
	}

	tx := r.db.Begin()

	order := models.Order{
		UserID:    data.User.ID,
		ProductID: data.JSON.ProductID,
		Quantity:  data.JSON.Quantity,
		Amount:    data.OrderAmount,
		Status:    "success",
		CreatedAt: time.Now().UTC(),
	}
	if createErr := tx.Create(&order).Error; createErr != nil {
		tx.Rollback()
		responses.ErrorResponse(c, http.StatusInternalServerError, createErr.Error())
		return
	}

	if transErr := r.createOrderTransactionsAndMovements(tx, data.User, data.Product, data.JSON, order, data.OrderAmount); transErr != nil {
		return
	}

	if commitErr := tx.Commit().Error; commitErr != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, commitErr.Error())
		return
	}

	newBalance := data.Balance - data.OrderAmount
	orderResponse := models.OrderSlimResponse(order)
	response := BuyResponse{
		Order:        orderResponse,
		ProductName:  data.Product.Name,
		ProductPrice: data.Product.Price,
		Balance:      newBalance,
	}

	responses.SuccessResponse(c, http.StatusOK, response)
}

func (r *Router) createOrderTransactionsAndMovements(
	tx *gorm.DB,
	user models.BotUser,
	product models.Product,
	jsonPayload OrderCreate,
	order models.Order,
	orderAmount float64,
) error {
	purchaseTransaction := models.Transaction{
		UserID:      user.ID,
		OrderID:     &order.ID,
		Type:        models.Purchase,
		Amount:      -orderAmount,
		Description: "Purchase of " + product.Name,
		CreatedAt:   time.Now().UTC(),
	}
	if err := tx.Create(&purchaseTransaction).Error; err != nil {
		return err
	}

	saleMovement := models.StockMovement{
		OrderID:     &order.ID,
		ProductID:   product.ID,
		Type:        models.Sale,
		Quantity:    -jsonPayload.Quantity,
		Description: "Sale to user " + strconv.FormatUint(uint64(user.ID), 10),
		CreatedAt:   time.Now().UTC(),
	}
	if err := tx.Create(&saleMovement).Error; err != nil {
		return err
	}

	if jsonPayload.ReferralBotToken != nil {
		if err := r.handleReferralTransaction(tx, jsonPayload.ReferralBotToken, order, orderAmount); err != nil {
			return err
		}
	}
	return nil
}

func (r *Router) validateAndRetrieveOrderData(c *gin.Context) (OrderValidationData, error) {
	var jsonPayload OrderCreate
	if err := c.ShouldBindJSON(&jsonPayload); err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, err.Error())
		return OrderValidationData{}, err
	}

	var user models.BotUser
	if err := r.db.Where("telegram_id = ? AND is_deleted = ?", jsonPayload.UserID, false).First(&user).Error; err != nil {
		responses.ErrorResponse(c, http.StatusNotFound, "Bot user not found")
		return OrderValidationData{}, err
	}

	var product models.Product
	if err := r.db.First(&product, jsonPayload.ProductID).Error; err != nil {
		responses.ErrorResponse(c, http.StatusNotFound, "Product not found")
		return OrderValidationData{}, err
	}

	var stock int64
	if err := r.db.Model(&models.StockMovement{}).Where("product_id = ?", product.ID).Select("sum(quantity)").
		Row().Scan(&stock); err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return OrderValidationData{}, err
	}

	if stock <= 0 {
		responses.ErrorResponse(c, http.StatusBadRequest, "Product out of stock")
		return OrderValidationData{}, errors.New("product out of stock")
	}

	var balance float64
	if err := r.db.Model(&models.Transaction{}).Where("user_id = ?", user.ID).Select("sum(amount)").
		Row().Scan(&balance); err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return OrderValidationData{}, err
	}

	orderAmount := product.Price * float64(jsonPayload.Quantity)
	if balance < orderAmount {
		responses.ErrorResponse(c, http.StatusBadRequest, "Insufficient balance")
		return OrderValidationData{}, errors.New("insufficient balance")
	}

	return OrderValidationData{
		JSON:        jsonPayload,
		User:        user,
		Product:     product,
		Balance:     balance,
		OrderAmount: orderAmount,
	}, nil
}

const percentDenominator = 100

func (r *Router) handleReferralTransaction(tx *gorm.DB, referralBotToken *string, order models.Order, orderAmount float64) error {
	var refBot models.ReferralBot
	if err := tx.Where("bot_token = ?", *referralBotToken).First(&refBot).Error; err == nil && refBot.IsActive {
		var seller models.User
		if sellerErr := tx.First(&seller, refBot.SellerID).Error; sellerErr == nil &&
			seller.ReferralProgramEnabled && seller.ReferralPercentage > 0 {
			refShare := orderAmount * (seller.ReferralPercentage / percentDenominator)
			refTransaction := models.RefTransaction{
				RefOwnerID: refBot.OwnerID,
				SellerID:   seller.ID,
				OrderID:    order.ID,
				Amount:     orderAmount,
				RefShare:   refShare,
				CreatedAt:  time.Now().UTC(),
			}
			if createErr := tx.Create(&refTransaction).Error; createErr != nil {
				return createErr
			}
		}
	}
	return nil
}

func (r *Router) getOrdersHandler(c *gin.Context) {
	var response []models.OrderResponse
	if err := r.db.Table("orders").
		Select("orders.*, bot_users.telegram_id as user_telegram_id, products.name as product_name").
		Joins("join bot_users on bot_users.id = orders.user_id").
		Joins("join products on products.id = orders.product_id").
		Order("orders.created_at desc").
		Scan(&response).Error; err != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	responses.SuccessResponse(c, http.StatusOK, response)
}

func (r *Router) cancelOrderHandler(c *gin.Context) {
	var order models.Order
	if err := r.db.First(&order, c.Param("id")).Error; err != nil {
		responses.ErrorResponse(c, http.StatusNotFound, "Order not found")
		return
	}

	if order.Status == "cancelled" {
		responses.ErrorResponse(c, http.StatusBadRequest, "Order is already cancelled")
		return
	}

	tx := r.db.Begin()

	returnMovement := models.StockMovement{
		OrderID:     &order.ID,
		ProductID:   order.ProductID,
		Type:        models.Return,
		Quantity:    order.Quantity,
		Description: "Return for cancelled order " + c.Param("id"),
		CreatedAt:   time.Now().UTC(),
	}
	if err := tx.Create(&returnMovement).Error; err != nil {
		tx.Rollback()
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	refundTransaction := models.Transaction{
		UserID:      order.UserID,
		OrderID:     &order.ID,
		Type:        models.Deposit,
		Amount:      order.Amount,
		Description: "Refund for cancelled order " + c.Param("id"),
		CreatedAt:   time.Now().UTC(),
	}
	if err := tx.Create(&refundTransaction).Error; err != nil {
		tx.Rollback()
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	order.Status = "cancelled"
	if err := tx.Save(&order).Error; err != nil {
		tx.Rollback()
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	if err := tx.Commit().Error; err != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	responses.SuccessResponse(c, http.StatusOK, gin.H{"message": "Order cancelled successfully"})
}
