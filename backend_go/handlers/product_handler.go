package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
)

type ProductHandler struct {
	productService services.ProductService
}

func NewProductHandler(productService services.ProductService) *ProductHandler {
	return &ProductHandler{productService: productService}
}

func (h *ProductHandler) GetProductsHandler(c *gin.Context) {
	categoryIDs, _ := c.GetQueryArray("category_ids")
	products, err := h.productService.GetProducts(categoryIDs)
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, products)
}

type productCreatePayload struct {
	Name                   string  `json:"name" binding:"required"`
	CategoryID             uint    `json:"category_id" binding:"required"`
	Price                  float64 `json:"price" binding:"gte=0"`
	InitialStock           int     `json:"initial_stock" binding:"gte=0"`
	Type                   string  `json:"type" binding:"oneof=item subscription"`
	SubscriptionPeriodDays int     `json:"subscription_period_days" binding:"gte=0"`
}

func (h *ProductHandler) CreateProductHandler(c *gin.Context) {
	var json productCreatePayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	product, err := h.productService.CreateProduct(json.Name, json.CategoryID, json.Price, json.InitialStock, json.Type, json.SubscriptionPeriodDays)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusCreated, product)
}

func (h *ProductHandler) GetProductHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid product ID"})
		return
	}

	product, err := h.productService.GetProduct(uint(id))
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, product)
}

type productUpdatePayload struct {
	Name                   string  `json:"name" binding:"required"`
	CategoryID             uint    `json:"category_id" binding:"required"`
	Price                  float64 `json:"price" binding:"gte=0"`
	Type                   string  `json:"type" binding:"oneof=item subscription"`
	SubscriptionPeriodDays int     `json:"subscription_period_days" binding:"gte=0"`
}

func (h *ProductHandler) UpdateProductHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid product ID"})
		return
	}

	var json productUpdatePayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	productData := models.Product{
		Name:                   json.Name,
		CategoryID:             json.CategoryID,
		Price:                  json.Price,
		Type:                   json.Type,
		SubscriptionPeriodDays: json.SubscriptionPeriodDays,
	}

	updatedProduct, err := h.productService.UpdateProduct(uint(id), productData)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, updatedProduct)
}

func (h *ProductHandler) DeleteProductHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid product ID"})
		return
	}

	if err := h.productService.DeleteProduct(uint(id)); err != nil {
		c.Error(err)
		return
	}

	c.Status(http.StatusNoContent)
}

type stockMovementPayload struct {
	Type        models.StockMovementType `json:"type" binding:"required"`
	Quantity    int                      `json:"quantity" binding:"required"`
	Description string                   `json:"description"`
	OrderID     *uint                    `json:"order_id"`
}

func (h *ProductHandler) CreateStockMovementHandler(c *gin.Context) {
	productID, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid product ID"})
		return
	}

	var json stockMovementPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	movement, err := h.productService.CreateStockMovement(uint(productID), json.Type, json.Quantity, json.Description, json.OrderID)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusCreated, movement)
}