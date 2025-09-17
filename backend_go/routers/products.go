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

func ProductsRouter(router *gin.Engine) {
	auth := router.Group("/api/products")
	auth.Use(middleware.AuthMiddleware())
	{
		auth.GET("", getProductsHandler)
		auth.POST("", createProductHandler)
		auth.GET("/:id", getProductHandler)
		auth.PUT("/:id", updateProductHandler)
		auth.DELETE("/:id", deleteProductHandler)
		auth.POST("/:id/stock/movements", createStockMovementHandler)
	}
}

func getProductsHandler(c *gin.Context) {
	var products []models.Product
	query := db.DB

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
		db.DB.Model(&models.StockMovement{}).Where("product_id = ?", p.ID).Select("sum(quantity)").Row().Scan(&stock)
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

func createProductHandler(c *gin.Context) {
	var json ProductCreate
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	var category models.Category
	if err := db.DB.First(&category, json.CategoryID).Error; err != nil {
		errorResponse(c, http.StatusBadRequest, "Category not found")
		return
	}

	product := models.Product{
		Name:       json.Name,
		CategoryID: json.CategoryID,
		Price:      json.Price,
	}
	if err := db.DB.Create(&product).Error; err != nil {
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
	if err := db.DB.Create(&stockMovement).Error; err != nil {
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

func getProductHandler(c *gin.Context) {
	var product models.Product
	if err := db.DB.First(&product, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Product not found")
		return
	}

	var stock int64
	db.DB.Model(&models.StockMovement{}).Where("product_id = ?", product.ID).Select("sum(quantity)").Row().Scan(&stock)

	response := models.ProductResponse{
		ID:         product.ID,
		Name:       product.Name,
		Price:      product.Price,
		CategoryID: product.CategoryID,
		Stock:      int(stock),
	}

	successResponse(c, http.StatusOK, response)
}

func updateProductHandler(c *gin.Context) {
	var product models.Product
	if err := db.DB.First(&product, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Product not found")
		return
	}

	var json models.Product
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	var category models.Category
	if err := db.DB.First(&category, json.CategoryID).Error; err != nil {
		errorResponse(c, http.StatusBadRequest, "Category not found")
		return
	}

	if err := db.DB.Model(&product).Updates(json).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	var stock int64
	db.DB.Model(&models.StockMovement{}).Where("product_id = ?", product.ID).Select("sum(quantity)").Row().Scan(&stock)

	response := models.ProductResponse{
		ID:         product.ID,
		Name:       product.Name,
		Price:      product.Price,
		CategoryID: product.CategoryID,
		Stock:      int(stock),
	}

	successResponse(c, http.StatusOK, response)
}

func deleteProductHandler(c *gin.Context) {
	var product models.Product
	if err := db.DB.First(&product, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Product not found")
		return
	}

	if err := db.DB.Delete(&product).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	c.Status(http.StatusNoContent)
}

func createStockMovementHandler(c *gin.Context) {
	productID, err := strconv.Atoi(c.Param("id"))
	if err != nil {
		errorResponse(c, http.StatusBadRequest, "Invalid product ID")
		return
	}

	var product models.Product
	if err := db.DB.First(&product, productID).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Product not found")
		return
	}

	var json models.StockMovement
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}
	json.ProductID = uint(productID)
	json.CreatedAt = time.Now().UTC()

	if err := db.DB.Create(&json).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	successResponse(c, http.StatusCreated, json)
}
