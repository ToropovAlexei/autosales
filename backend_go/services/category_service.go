package services

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
)

// CategoryService определяет интерфейс для работы с категориями, включая иерархию.
type CategoryService interface {
	GetAll() ([]models.CategoryResponse, error)
	GetByID(id uint) (*models.CategoryResponse, error)
	Create(ctx *gin.Context, name string, parentID *uint, imageID *uuid.UUID) (*models.CategoryResponse, error)
	Update(ctx *gin.Context, id uint, name string, parentID *uint, imageID *uuid.UUID) (*models.CategoryResponse, error)
	Delete(ctx *gin.Context, id uint) error
}

type categoryService struct {
	categoryRepo   repositories.CategoryRepository
	productService ProductService
	auditLogService  AuditLogService
}

func NewCategoryService(categoryRepo repositories.CategoryRepository, productService ProductService, auditLogService AuditLogService) CategoryService {
	return &categoryService{categoryRepo: categoryRepo, productService: productService, auditLogService: auditLogService}
}

// buildCategoryTree строит иерархическое дерево из плоского списка категорий с помощью рекурсии.
func buildCategoryTree(categories []models.Category) []models.CategoryResponse {
	// Создаем карту, где ключ - это ID родителя, а значение - список его прямых дочерних категорий.
	categoryMap := make(map[uint][]models.Category)
	var roots []models.Category

	for _, c := range categories {
		if c.ParentID == nil {
			roots = append(roots, c)
		} else {
			categoryMap[*c.ParentID] = append(categoryMap[*c.ParentID], c)
		}
	}

	// Рекурсивная функция для построения дерева ответов.
	var buildResponse func([]models.Category) []models.CategoryResponse
	buildResponse = func(cats []models.Category) []models.CategoryResponse {
		if len(cats) == 0 {
			return []models.CategoryResponse{}
		}
		res := make([]models.CategoryResponse, len(cats))
		for i, c := range cats {
			responseItem := models.CategoryResponse{
				ID:            c.ID,
				Name:          c.Name,
				ParentID:      c.ParentID,
				SubCategories: buildResponse(categoryMap[c.ID]), // Рекурсивный вызов для дочерних элементов
				ImageID:       c.ImageID,
			}
			res[i] = responseItem
		}
		return res
	}

	return buildResponse(roots)
}

func (s *categoryService) GetAll() ([]models.CategoryResponse, error) {
	categories, err := s.categoryRepo.GetAll()
	if err != nil {
		return nil, apperrors.New(500, "Failed to get all categories", err)
	}

	return buildCategoryTree(categories), nil
}

func (s *categoryService) GetByID(id uint) (*models.CategoryResponse, error) {
	category, err := s.categoryRepo.GetByID(id)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "Category", ID: id}
	}

	return &models.CategoryResponse{
		ID:       category.ID,
		Name:     category.Name,
		ParentID: category.ParentID,
	}, nil
}

func (s *categoryService) Create(ctx *gin.Context, name string, parentID *uint, imageID *uuid.UUID) (*models.CategoryResponse, error) {
	if parentID != nil && *parentID == 0 {
		parentID = nil
	}
	category := &models.Category{Name: name, ParentID: parentID, ImageID: imageID}
	if err := s.categoryRepo.Create(category); err != nil {
		return nil, apperrors.New(500, "Failed to create category", err)
	}

	response := &models.CategoryResponse{
		ID:       category.ID,
		Name:     category.Name,
		ParentID: category.ParentID,
	}

	s.auditLogService.Log(ctx, "CATEGORY_CREATE", "Category", category.ID, map[string]interface{}{"after": response})

	return response, nil
}

func (s *categoryService) Update(ctx *gin.Context, id uint, name string, parentID *uint, imageID *uuid.UUID) (*models.CategoryResponse, error) {
	if parentID != nil && *parentID == 0 {
		parentID = nil
	}
	before, err := s.GetByID(id)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "Category", ID: id}
	}

	category, err := s.categoryRepo.GetByID(id)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "Category", ID: id}
	}

	updateData := models.Category{Name: name, ParentID: parentID, ImageID: imageID}
	if err := s.categoryRepo.Update(category, updateData); err != nil {
		return nil, apperrors.New(500, "Failed to update category", err)
	}

	after := &models.CategoryResponse{
		ID:       id,
		Name:     name,
		ParentID: parentID,
	}

	s.auditLogService.Log(ctx, "CATEGORY_UPDATE", "Category", id, map[string]interface{}{"before": before, "after": after})

	return after, nil
}

func (s *categoryService) Delete(ctx *gin.Context, id uint) error {
	before, err := s.GetByID(id)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "Category", ID: id}
	}

	category, err := s.categoryRepo.GetByID(id)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "Category", ID: id}
	}
	if err := s.categoryRepo.Delete(category); err != nil {
		return apperrors.New(500, "Failed to delete category", err)
	}

	s.auditLogService.Log(ctx, "CATEGORY_DELETE", "Category", id, map[string]interface{}{"before": before})

	return nil
}
