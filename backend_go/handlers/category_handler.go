package handlers

import (
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"

	"github.com/gin-gonic/gin"
)

type CategoryHandler struct {
	categoryService services.CategoryService
}

func NewCategoryHandler(categoryService services.CategoryService) *CategoryHandler {
	return &CategoryHandler{categoryService: categoryService}
}

// @Summary      Get all categories
// @Description  get all categories
// @Tags         categories
// @Accept       json
// @Produce      json
// @Success      200  {object}  responses.ResponseSchema[[]models.CategoryResponse]
// @Failure      500  {object}  responses.ErrorResponseSchema
// @Router       /categories [get]
func (h *CategoryHandler) GetCategoriesHandler(c *gin.Context) {
	categories, err := h.categoryService.GetAll()
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, categories)
}

type categoryPayload struct {
	Name string `json:"name" binding:"required"`
}

// @Summary      Create a new category
// @Description  create a new category with the given name
// @Tags         categories
// @Accept       json
// @Produce      json
// @Param        category  body      categoryPayload  true  "Category Name"
// @Success      201      {object}  responses.ResponseSchema[models.CategoryResponse]
// @Failure      400      {object}  responses.ErrorResponseSchema
// @Failure      500      {object}  responses.ErrorResponseSchema
// @Router       /categories [post]
func (h *CategoryHandler) CreateCategoryHandler(c *gin.Context) {
	var json categoryPayload
	if err := bindJSON(c, &json); err != nil {
		c.Error(err)
		return
	}

	category, err := h.categoryService.Create(json.Name)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusCreated, category)
}

// @Summary      Get a category by ID
// @Description  get a single category by its ID
// @Tags         categories
// @Accept       json
// @Produce      json
// @Param        id   path      int  true  "Category ID"
// @Success      200  {object}  responses.ResponseSchema[models.CategoryResponse]
// @Failure      400      {object}  responses.ErrorResponseSchema
// @Failure      404      {object}  responses.ErrorResponseSchema
// @Router       /categories/{id} [get]
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
// @Description  update a category's name by its ID
// @Tags         categories
// @Accept       json
// @Produce      json
// @Param        id   path      int              true  "Category ID"
// @Param        category  body      categoryPayload  true  "New Category Name"
// @Success      200  {object}  responses.ResponseSchema[models.CategoryResponse]
// @Failure      400      {object}  responses.ErrorResponseSchema
// @Failure      500      {object}  responses.ErrorResponseSchema
// @Router       /categories/{id} [put]
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

	category, err := h.categoryService.Update(id, json.Name)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, category)
}

// @Summary      Delete a category
// @Description  delete a category by its ID
// @Tags         categories
// @Accept       json
// @Produce      json
// @Param        id   path      int  true  "Category ID"
// @Success      204  {object}  nil
// @Failure      400      {object}  responses.ErrorResponseSchema
// @Failure      500      {object}  responses.ErrorResponseSchema
// @Router       /categories/{id} [delete]
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