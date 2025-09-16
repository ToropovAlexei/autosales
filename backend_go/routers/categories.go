package routers

import (
	"net/http"

	"frbktg/backend_go/db"
	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
)

func CategoriesRouter(router *gin.Engine) {
	api := router.Group("/api/categories")
	api.Use(middleware.AuthMiddleware())
	{
		api.GET("", getCategoriesHandler)
		api.POST("", createCategoryHandler)
		api.GET("/:id", getCategoryHandler)
		api.PUT("/:id", updateCategoryHandler)
		api.DELETE("/:id", deleteCategoryHandler)
	}
}

func getCategoriesHandler(c *gin.Context) {
	var categories []models.Category
	if err := db.DB.Find(&categories).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	var response []models.CategoryResponse
	for _, category := range categories {
		response = append(response, models.CategoryResponse{
			ID:   category.ID,
			Name: category.Name,
		})
	}

	successResponse(c, http.StatusOK, response)
}

func createCategoryHandler(c *gin.Context) {
	var json models.Category
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	if err := db.DB.Create(&json).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	response := models.CategoryResponse{
		ID:   json.ID,
		Name: json.Name,
	}

	successResponse(c, http.StatusCreated, response)
}

func getCategoryHandler(c *gin.Context) {
	var category models.Category
	if err := db.DB.First(&category, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Category not found")
		return
	}

	response := models.CategoryResponse{
		ID:   category.ID,
		Name: category.Name,
	}

	successResponse(c, http.StatusOK, response)
}

func updateCategoryHandler(c *gin.Context) {
	var category models.Category
	if err := db.DB.First(&category, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Category not found")
		return
	}

	var json models.Category
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	if err := db.DB.Model(&category).Updates(json).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	response := models.CategoryResponse{
		ID:   category.ID,
		Name: category.Name,
	}

	successResponse(c, http.StatusOK, response)
}

func deleteCategoryHandler(c *gin.Context) {
	var category models.Category
	if err := db.DB.First(&category, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Category not found")
		return
	}

	if err := db.DB.Delete(&category).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}
	c.Status(http.StatusNoContent)
}
