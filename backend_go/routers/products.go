package routers

import (
	"errors"
	"net/http"
	"strconv"
	"time"

	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
)

func (r *Router) ProductsRouter(router *gin.Engine) {
	auth := router.Group("/api/products")
	auth.Use(middleware.AuthMiddleware(r.appSettings, r.db))
	{
		auth.GET("", r.getProductsHandler)
		auth.POST("", r.createProductHandler)
		auth.GET("/:id", r.getProductHandler)
		auth.PUT("/:id", r.updateProductHandler)
		auth.DELETE("/:id", r.deleteProductHandler)
		auth.POST("/:id/stock/movements", r.createStockMovementHandler)
	}
}

func (r *Router) getProductsHandler(c *gin.Context) {
	var products []models.Product
	query := r.db

	categoryIDs, ok := c.GetQueryArray("category_ids")
	if ok {
		query = query.Where("category_id IN ?", categoryIDs)
	}

	if err := query.Find(&products).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	var response []models.ProductResponse
	for _, p := range products {
		var stock int64
		if err := r.db.Model(&models.StockMovement{}).Where("product_id = ?", p.ID).Select("sum(quantity)").
			Row().Scan(&stock); err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
			errorResponse(c, http.StatusInternalServerError, err.Error())
			return
		}
		response = append(response, models.ProductResponse{
			ID:         p.ID,
			Name:       p.Name,
			Price:      p.Price,
			CategoryID: p.CategoryID,
			Stock:      int(stock),
		})
	}

	successResponse(c, http.StatusOK, response)
}

type ProductCreate struct {
	Name         string  `json:"name"`
	CategoryID   uint    `json:"category_id"`
	Price        float64 `json:"price"`
	InitialStock int     `json:"initial_stock"`
}

func (r *Router) createProductHandler(c *gin.Context) {
	var json ProductCreate
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	var category models.Category
	if err := r.db.First(&category, json.CategoryID).Error; err != nil {
		errorResponse(c, http.StatusBadRequest, "Category not found")
		return
	}

	product := models.Product{
		Name:       json.Name,
		CategoryID: json.CategoryID,
		Price:      json.Price,
	}
	if err := r.db.Create(&product).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	stockMovement := models.StockMovement{
		ProductID:   product.ID,
		Type:        models.Initial,
		Quantity:    json.InitialStock,
		Description: "Initial stock",
		CreatedAt:   time.Now().UTC(),
	}
	if err := r.db.Create(&stockMovement).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	response := models.ProductResponse{
		ID:         product.ID,
		Name:       product.Name,
		Price:      product.Price,
		CategoryID: product.CategoryID,
		Stock:      json.InitialStock,
	}

	successResponse(c, http.StatusCreated, response)
}

func (r *Router) getProductHandler(c *gin.Context) {
	var product models.Product
	if err := r.db.First(&product, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Product not found")
		return
	}

	var stock int64
	if err := r.db.Model(&models.StockMovement{}).Where("product_id = ?", product.ID).Select("sum(quantity)").
			Row().Scan(&stock); err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	response := models.ProductResponse{
		ID:         product.ID,
		Name:       product.Name,
		Price:      product.Price,
		CategoryID: product.CategoryID,
		Stock:      int(stock),
	}

	successResponse(c, http.StatusOK, response)
}

func (r *Router) updateProductHandler(c *gin.Context) {
	var product models.Product
	if err := r.db.First(&product, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Product not found")
		return
	}

	var json models.Product
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	var category models.Category
	if err := r.db.First(&category, json.CategoryID).Error; err != nil {
		errorResponse(c, http.StatusBadRequest, "Category not found")
		return
	}

	if err := r.db.Model(&product).Updates(json).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	var stock int64
	if err := r.db.Model(&models.StockMovement{}).Where("product_id = ?", product.ID).Select("sum(quantity)").
			Row().Scan(&stock); err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	response := models.ProductResponse{
		ID:         product.ID,
		Name:       product.Name,
		Price:      product.Price,
		CategoryID: product.CategoryID,
		Stock:      int(stock),
	}

	successResponse(c, http.StatusOK, response)
}

func (r *Router) deleteProductHandler(c *gin.Context) {
	var product models.Product
	if err := r.db.First(&product, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Product not found")
		return
	}

	if err := r.db.Delete(&product).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	c.Status(http.StatusNoContent)
}

func (r *Router) createStockMovementHandler(c *gin.Context) {
	productID, err := strconv.Atoi(c.Param("id"))
	if err != nil || productID < 0 {
		errorResponse(c, http.StatusBadRequest, "Invalid product ID")
		return
	}

	var product models.Product
	if findErr := r.db.First(&product, productID).Error; findErr != nil {
		errorResponse(c, http.StatusNotFound, "Product not found")
		return
	}

	var json models.StockMovement
	if bindErr := c.ShouldBindJSON(&json); bindErr != nil {
		errorResponse(c, http.StatusBadRequest, bindErr.Error())
		return
	}
	json.ProductID = uint(productID)
	json.CreatedAt = time.Now().UTC()

	if createErr := r.db.Create(&json).Error; createErr != nil {
		errorResponse(c, http.StatusInternalServerError, createErr.Error())
		return
	}

	successResponse(c, http.StatusCreated, json)
}
