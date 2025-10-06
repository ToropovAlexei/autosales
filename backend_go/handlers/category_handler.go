package handlers

import (
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
)

type CategoryHandler struct {
	categoryService services.CategoryService
}

func NewCategoryHandler(categoryService services.CategoryService) *CategoryHandler {
	return &CategoryHandler{categoryService: categoryService}
}

// @Summary      Get all categories
// @Description  Get a hierarchical list of all categories.
// @Tags         Categories
// @Produce      json
// @Success      200  {object}  responses.ResponseSchema[[]models.CategoryResponse]
// @Failure      500  {object}  responses.ErrorResponseSchema
// @Router       /categories [get]
// @Security     ApiKeyAuth
// @Security     ServiceApiKeyAuth
func (h *CategoryHandler) GetCategoriesHandler(c *gin.Context) {
	categories, err := h.categoryService.GetAll()
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, categories)
}

type categoryPayload struct {
	Name     string     `json:"name" binding:"required"`
	ParentID *uint      `json:"parent_id"`
	ImageID  *uuid.UUID `json:"image_id"`
}

// @Summary      Create a new category
// @Description  Create a new category with the given name and optional parent_id.
// @Tags         Categories
// @Accept       json
// @Produce      json
// @Param        category  body      categoryPayload  true  "Category Name and Parent ID"
// @Success      201      {object}  responses.ResponseSchema[models.CategoryResponse]
// @Failure      400      {object}  responses.ErrorResponseSchema
// @Failure      500      {object}  responses.ErrorResponseSchema
// @Router       /categories [post]
// @Security     ApiKeyAuth
func (h *CategoryHandler) CreateCategoryHandler(c *gin.Context) {
	var json categoryPayload
	if err := bindJSON(c, &json); err != nil {
		c.Error(err)
		return
	}

	category, err := h.categoryService.Create(json.Name, json.ParentID, json.ImageID)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusCreated, category)
}

// @Summary      Get a category by ID
// @Description  Get a single category by its ID.
// @Tags         Categories
// @Produce      json
// @Param        id   path      int  true  "Category ID"
// @Success      200  {object}  responses.ResponseSchema[models.CategoryResponse]
// @Failure      400      {object}  responses.ErrorResponseSchema
// @Failure      404      {object}  responses.ErrorResponseSchema
// @Router       /categories/{id} [get]
// @Security     ApiKeyAuth
func (h *CategoryHandler) GetCategoryHandler(c *gin.Context) {
	id, err := getIDFromParam(c)
	if err != nil {
		c.Error(err)
		return
	}

	category, err := h.categoryService.GetByID(id)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, category)
}

// @Summary      Update a category
// @Description  Update a category's name and/or parent by its ID.
// @Tags         Categories
// @Accept       json
// @Produce      json
// @Param        id   path      int              true  "Category ID"
// @Param        category  body      categoryPayload  true  "New Category Name and/or Parent ID"
// @Success      200  {object}  responses.ResponseSchema[models.CategoryResponse]
// @Failure      400      {object}  responses.ErrorResponseSchema
// @Failure      404      {object}  responses.ErrorResponseSchema
// @Failure      500      {object}  responses.ErrorResponseSchema
// @Router       /categories/{id} [put]
// @Security     ApiKeyAuth
func (h *CategoryHandler) UpdateCategoryHandler(c *gin.Context) {
	id, err := getIDFromParam(c)
	if err != nil {
		c.Error(err)
		return
	}

	var json categoryPayload
	if err := bindJSON(c, &json); err != nil {
		c.Error(err)
		return
	}

	category, err := h.categoryService.Update(id, json.Name, json.ParentID, json.ImageID)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, category)
}

// @Summary      Delete a category
// @Description  Delete a category by its ID.
// @Tags         Categories
// @Produce      json
// @Param        id   path      int  true  "Category ID"
// @Success      204  {object}  nil
// @Failure      400      {object}  responses.ErrorResponseSchema
// @Failure      404      {object}  responses.ErrorResponseSchema
// @Failure      500      {object}  responses.ErrorResponseSchema
// @Router       /categories/{id} [delete]
// @Security     ApiKeyAuth
func (h *CategoryHandler) DeleteCategoryHandler(c *gin.Context) {
	id, err := getIDFromParam(c)
	if err != nil {
		c.Error(err)
		return
	}

	if err := h.categoryService.Delete(id); err != nil {
		c.Error(err)
		return
	}

	c.Status(http.StatusNoContent)
}
