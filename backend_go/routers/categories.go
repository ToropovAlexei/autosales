package routers

import (
	"net/http"

	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
)

func (r *Router) CategoriesRouter(router *gin.Engine) {
	api := router.Group("/api/categories")
	api.Use(middleware.AuthMiddleware(r.appSettings, r.db))
	api.GET("", r.getCategoriesHandler)
	api.POST("", r.createCategoryHandler)
	api.GET("/:id", r.getCategoryHandler)
	api.PUT("/:id", r.updateCategoryHandler)
	api.DELETE("/:id", r.deleteCategoryHandler)
}

func (r *Router) getCategoriesHandler(c *gin.Context) {
	var categories []models.Category
	if err := r.db.Find(&categories).Error; err != nil {
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

func (r *Router) createCategoryHandler(c *gin.Context) {
	var json models.Category
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	if err := r.db.Create(&json).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	response := models.CategoryResponse{
		ID:   json.ID,
		Name: json.Name,
	}

	successResponse(c, http.StatusCreated, response)
}

func (r *Router) getCategoryHandler(c *gin.Context) {
	var category models.Category
	if err := r.db.First(&category, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Category not found")
		return
	}

	response := models.CategoryResponse{
		ID:   category.ID,
		Name: category.Name,
	}

	successResponse(c, http.StatusOK, response)
}

func (r *Router) updateCategoryHandler(c *gin.Context) {
	var category models.Category
	if err := r.db.First(&category, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Category not found")
		return
	}

	var json models.Category
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	if err := r.db.Model(&category).Updates(json).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	response := models.CategoryResponse{
		ID:   category.ID,
		Name: category.Name,
	}

	successResponse(c, http.StatusOK, response)
}

func (r *Router) deleteCategoryHandler(c *gin.Context) {
	var category models.Category
	if err := r.db.First(&category, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Category not found")
		return
	}

	if err := r.db.Delete(&category).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}
	c.Status(http.StatusNoContent)
}
