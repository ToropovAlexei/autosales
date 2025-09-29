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

// @Summary      List products
// @Description  Get a list of all products, optionally filtered by category.
// @Tags         Products
// @Produce      json
// @Param        category_ids[] query []string false "Category IDs to filter by"
// @Success      200 {object} responses.ResponseSchema[[]models.ProductResponse]
// @Failure      500 {object} responses.ErrorResponseSchema
// @Router       /products [get]
// @Security     ApiKeyAuth
// @Security     ServiceApiKeyAuth
func (h *ProductHandler) GetProductsHandler(c *gin.Context) {
	categoryIDStrs, _ := c.GetQueryArray("category_ids[]")
	var categoryIDs []uint
	for _, s := range categoryIDStrs {
		id, err := strconv.ParseUint(s, 10, 32)
		if err == nil {
			categoryIDs = append(categoryIDs, uint(id))
		}
	}

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

// @Summary      Create a product
// @Description  Adds a new product to the store.
// @Tags         Products
// @Accept       json
// @Produce      json
// @Param        product body productCreatePayload true "Product data"
// @Success      201 {object} responses.ResponseSchema[models.ProductResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      500 {object} responses.ErrorResponseSchema
// @Router       /products [post]
// @Security     ApiKeyAuth
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

// @Summary      Get a product by ID
// @Description  Retrieves details for a single product.
// @Tags         Products
// @Produce      json
// @Param        id path int true "Product ID"
// @Success      200 {object} responses.ResponseSchema[models.ProductResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Router       /products/{id} [get]
// @Security     ApiKeyAuth
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

// @Summary      Update a product
// @Description  Updates an existing product.
// @Tags         Products
// @Accept       json
// @Produce      json
// @Param        id path int true "Product ID"
// @Param        product body productUpdatePayload true "Product data"
// @Success      200 {object} responses.ResponseSchema[models.ProductResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Failure      500 {object} responses.ErrorResponseSchema
// @Router       /products/{id} [put]
// @Security     ApiKeyAuth
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

// @Summary      Delete a product
// @Description  Deletes a product by its ID.
// @Tags         Products
// @Param        id path int true "Product ID"
// @Success      204
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Router       /products/{id} [delete]
// @Security     ApiKeyAuth
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

// @Summary      Create a stock movement
// @Description  Adds a stock movement (e.g., initial stock, sale, return) for a product.
// @Tags         Products, Stock
// @Accept       json
// @Produce      json
// @Param        id path int true "Product ID"
// @Param        movement body stockMovementPayload true "Stock movement data"
// @Success      201 {object} responses.ResponseSchema[models.StockMovement]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Failure      500 {object} responses.ErrorResponseSchema
// @Router       /products/{id}/stock/movements [post]
// @Security     ApiKeyAuth
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