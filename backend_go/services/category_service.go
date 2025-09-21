package services

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
)

// CategoryService определяет интерфейс для работы с категориями, включая иерархию.
type CategoryService interface {
	GetAll() ([]models.CategoryResponse, error)
	GetByID(id uint) (*models.CategoryResponse, error)
	Create(name string, parentID *uint) (*models.CategoryResponse, error)
	Update(id uint, name string, parentID *uint) (*models.CategoryResponse, error)
	Delete(id uint) error
}

type categoryService struct {
	categoryRepo repositories.CategoryRepository
}

func NewCategoryService(categoryRepo repositories.CategoryRepository) CategoryService {
	return &categoryService{categoryRepo: categoryRepo}
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
			res[i] = models.CategoryResponse{
				ID:            c.ID,
				Name:          c.Name,
				ParentID:      c.ParentID,
				SubCategories: buildResponse(categoryMap[c.ID]), // Рекурсивный вызов для дочерних элементов
			}
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

func (s *categoryService) Create(name string, parentID *uint) (*models.CategoryResponse, error) {
	category := &models.Category{Name: name, ParentID: parentID}
	if err := s.categoryRepo.Create(category); err != nil {
		return nil, apperrors.New(500, "Failed to create category", err)
	}
	return &models.CategoryResponse{
		ID:       category.ID,
		Name:     category.Name,
		ParentID: category.ParentID,
	}, nil
}

func (s *categoryService) Update(id uint, name string, parentID *uint) (*models.CategoryResponse, error) {
	category, err := s.categoryRepo.GetByID(id)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Base: apperrors.New(404, "", err), Resource: "Category", ID: id}
	}

	updateData := models.Category{Name: name, ParentID: parentID}
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
