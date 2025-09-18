package handlers

import (
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
)

type CategoryHandler struct {
	categoryService services.CategoryService
}

func NewCategoryHandler(categoryService services.CategoryService) *CategoryHandler {
	return &CategoryHandler{categoryService: categoryService}
}

func (h *CategoryHandler) GetCategoriesHandler(c *gin.Context) {
	categories, err := h.categoryService.GetAll()
	if err != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}
	responses.SuccessResponse(c, http.StatusOK, categories)
}

type categoryPayload struct {
	Name string `json:"name" binding:"required"`
}

func (h *CategoryHandler) CreateCategoryHandler(c *gin.Context) {
	var json categoryPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	category, err := h.categoryService.Create(json.Name)
	if err != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	responses.SuccessResponse(c, http.StatusCreated, category)
}

func (h *CategoryHandler) GetCategoryHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, "Invalid category ID")
		return
	}

	category, err := h.categoryService.GetByID(uint(id))
	if err != nil {
		responses.ErrorResponse(c, http.StatusNotFound, "Category not found")
		return
	}

	responses.SuccessResponse(c, http.StatusOK, category)
}

func (h *CategoryHandler) UpdateCategoryHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, "Invalid category ID")
		return
	}

	var json categoryPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	category, err := h.categoryService.Update(uint(id), json.Name)
	if err != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	responses.SuccessResponse(c, http.StatusOK, category)
}

func (h *CategoryHandler) DeleteCategoryHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, "Invalid category ID")
		return
	}

	if err := h.categoryService.Delete(uint(id)); err != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	c.Status(http.StatusNoContent)
}
