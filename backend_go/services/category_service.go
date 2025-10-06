package services

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"

	"github.com/google/uuid"
)

// CategoryService определяет интерфейс для работы с категориями, включая иерархию.
type CategoryService interface {
	GetAll() ([]models.CategoryResponse, error)
	GetByID(id uint) (*models.CategoryResponse, error)
	Create(name string, parentID *uint, imageID *uuid.UUID) (*models.CategoryResponse, error)
	Update(id uint, name string, parentID *uint, imageID *uuid.UUID) (*models.CategoryResponse, error)
	Delete(id uint) error
}

type categoryService struct {
	categoryRepo   repositories.CategoryRepository
	productService ProductService
}

func NewCategoryService(categoryRepo repositories.CategoryRepository, productService ProductService) CategoryService {
	return &categoryService{categoryRepo: categoryRepo, productService: productService}
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
	// Sync external products and categories first to ensure the category list is up-to-date.
	if err := s.productService.SyncExternalProductsAndCategories(); err != nil {
		// We can log this error but we don't want to fail the whole request
		// as the user can still see the internal categories.
		// slog.Error("failed to sync external products for categories", "error", err)
	}

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

func (s *categoryService) Create(name string, parentID *uint, imageID *uuid.UUID) (*models.CategoryResponse, error) {
	if parentID != nil && *parentID == 0 {
		parentID = nil
	}
	category := &models.Category{Name: name, ParentID: parentID, ImageID: imageID}
	if err := s.categoryRepo.Create(category); err != nil {
		return nil, apperrors.New(500, "Failed to create category", err)
	}
	return &models.CategoryResponse{
		ID:       category.ID,
		Name:     category.Name,
		ParentID: category.ParentID,
	}, nil
}

func (s *categoryService) Update(id uint, name string, parentID *uint, imageID *uuid.UUID) (*models.CategoryResponse, error) {
	if parentID != nil && *parentID == 0 {
		parentID = nil
	}
	category, err := s.categoryRepo.GetByID(id)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "Category", ID: id}
	}

	updateData := models.Category{Name: name, ParentID: parentID, ImageID: imageID}
	if err := s.categoryRepo.Update(category, updateData); err != nil {
		return nil, apperrors.New(500, "Failed to update category", err)
	}

	return &models.CategoryResponse{
		ID:       id,
		Name:     name,
		ParentID: parentID,
	}, nil
}

func (s *categoryService) Delete(id uint) error {
	category, err := s.categoryRepo.GetByID(id)
	if err != nil {
		return &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "Category", ID: id}
	}
	if err := s.categoryRepo.Delete(category); err != nil {
		return apperrors.New(500, "Failed to delete category", err)
	}
	return nil
}
