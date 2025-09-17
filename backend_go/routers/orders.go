package routers

import (
	"net/http"
	"strconv"
	"time"

	"frbktg/backend_go/db"
	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"
	

	"github.com/gin-gonic/gin"
)

func OrdersRouter(router *gin.Engine) {
	service := router.Group("/api/orders")
	service.Use(middleware.ServiceTokenMiddleware())
	{
		service.POST("/buy-from-balance", buyFromBalanceHandler)
	}

	auth := router.Group("/api/orders")
	auth.Use(middleware.AuthMiddleware())
	{
		auth.GET("", getOrdersHandler)
		auth.POST("/:id/cancel", cancelOrderHandler)
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
	ProductName  string                      `json:"product_name"`
	ProductPrice float64                     `json:"product_price"`
	Balance      float64                     `json:"balance"`
}

func buyFromBalanceHandler(c *gin.Context) {
	var json OrderCreate
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	var user models.BotUser
	if err := db.DB.Where("telegram_id = ? AND is_deleted = ?", json.UserID, false).First(&user).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Bot user not found")
		return
	}

	var product models.Product
	if err := db.DB.First(&product, json.ProductID).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Product not found")
		return
	}

	var stock int64
	db.DB.Model(&models.StockMovement{}).Where("product_id = ?", product.ID).Select("sum(quantity)").Row().Scan(&stock)

	if stock <= 0 {
		errorResponse(c, http.StatusBadRequest, "Product out of stock")
		return
	}

	var balance float64
	db.DB.Model(&models.Transaction{}).Where("user_id = ?", user.ID).Select("sum(amount)").Row().Scan(&balance)

	orderAmount := product.Price * float64(json.Quantity)
	if balance < orderAmount {
		errorResponse(c, http.StatusBadRequest, "Insufficient balance")
		return
	}

	tx := db.DB.Begin()

	order := models.Order{
		UserID:    user.ID,
		ProductID: json.ProductID,
		Quantity:  json.Quantity,
		Amount:    orderAmount,
		Status:    "success",
		CreatedAt: time.Now().UTC(),
	}
	if err := tx.Create(&order).Error; err != nil {
		tx.Rollback()
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	purchaseTransaction := models.Transaction{
		UserID:      user.ID,
		OrderID:     &order.ID,
		Type:        models.Purchase,
		Amount:      -orderAmount,
		Description: "Purchase of " + product.Name,
		CreatedAt:   time.Now().UTC(),
	}
	if err := tx.Create(&purchaseTransaction).Error; err != nil {
		tx.Rollback()
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	saleMovement := models.StockMovement{
		OrderID:     &order.ID,
		ProductID:   product.ID,
		Type:        models.Sale,
		Quantity:    -json.Quantity,
		Description: "Sale to user " + strconv.FormatUint(uint64(user.ID), 10),
		CreatedAt:   time.Now().UTC(),
	}
	if err := tx.Create(&saleMovement).Error; err != nil {
		tx.Rollback()
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	if json.ReferralBotToken != nil {
		var refBot models.ReferralBot
		if err := tx.Where("bot_token = ?", *json.ReferralBotToken).First(&refBot).Error; err == nil && refBot.IsActive {
			var seller models.User
			if err := tx.First(&seller, refBot.SellerID).Error; err == nil && seller.ReferralProgramEnabled && seller.ReferralPercentage > 0 {
				refShare := orderAmount * (seller.ReferralPercentage / 100)
				refTransaction := models.RefTransaction{
					RefOwnerID: refBot.OwnerID,
					SellerID:   seller.ID,
					OrderID:    order.ID,
					Amount:     orderAmount,
					RefShare:   refShare,
					CreatedAt:  time.Now().UTC(),
				}
				if err := tx.Create(&refTransaction).Error; err != nil {
					tx.Rollback()
					errorResponse(c, http.StatusInternalServerError, err.Error())
					return
				}
			}
		}
	}

	if err := tx.Commit().Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	newBalance := balance - orderAmount
	orderResponse := models.OrderSlimResponse{
		ID:        order.ID,
		UserID:    order.UserID,
		ProductID: order.ProductID,
		Quantity:  order.Quantity,
		Amount:    order.Amount,
		Status:    order.Status,
		CreatedAt: order.CreatedAt,
	}
	response := BuyResponse{
		Order:        orderResponse,
		ProductName:  product.Name,
		ProductPrice: product.Price,
		Balance:      newBalance,
	}

	successResponse(c, http.StatusOK, response)
}

func getOrdersHandler(c *gin.Context) {
	var response []models.OrderResponse
	if err := db.DB.Table("orders").
		Select("orders.*, bot_users.telegram_id as user_telegram_id, products.name as product_name").
		Joins("join bot_users on bot_users.id = orders.user_id").
		Joins("join products on products.id = orders.product_id").
		Order("orders.created_at desc").
		Scan(&response).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	successResponse(c, http.StatusOK, response)
}

func cancelOrderHandler(c *gin.Context) {
	var order models.Order
	if err := db.DB.First(&order, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Order not found")
		return
	}

	if order.Status == "cancelled" {
		errorResponse(c, http.StatusBadRequest, "Order is already cancelled")
		return
	}

	tx := db.DB.Begin()

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
		errorResponse(c, http.StatusInternalServerError, err.Error())
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
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	order.Status = "cancelled"
	if err := tx.Save(&order).Error; err != nil {
		tx.Rollback()
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	if err := tx.Commit().Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	successResponse(c, http.StatusOK, gin.H{"message": "Order cancelled successfully"})
}
