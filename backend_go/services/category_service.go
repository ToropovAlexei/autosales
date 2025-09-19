package services

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
)

type CategoryService interface {
	GetAll() ([]models.CategoryResponse, error)
	GetByID(id uint) (*models.CategoryResponse, error)
	Create(name string) (*models.CategoryResponse, error)
	Update(id uint, name string) (*models.CategoryResponse, error)
	Delete(id uint) error
}

type categoryService struct {
	categoryRepo repositories.CategoryRepository
}

func NewCategoryService(categoryRepo repositories.CategoryRepository) CategoryService {
	return &categoryService{categoryRepo: categoryRepo}
}

func (s *categoryService) GetAll() ([]models.CategoryResponse, error) {
	categories, err := s.categoryRepo.GetAll()
	if err != nil {
		return nil, err
	}

	var response []models.CategoryResponse
	for _, category := range categories {
		response = append(response, models.CategoryResponse{
			ID:   category.ID,
			Name: category.Name,
		})
	}

	return response, nil
}

func (s *categoryService) GetByID(id uint) (*models.CategoryResponse, error) {
	category, err := s.categoryRepo.GetByID(id)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Resource: "Category", ID: id}
	}

	return &models.CategoryResponse{
		ID:   category.ID,
		Name: category.Name,
	}, nil
}

func (s *categoryService) Create(name string) (*models.CategoryResponse, error) {
	category := &models.Category{Name: name}
	if err := s.categoryRepo.Create(category); err != nil {
		return nil, err
	}
	return &models.CategoryResponse{
		ID:   category.ID,
		Name: category.Name,
	}, nil
}

func (s *categoryService) Update(id uint, name string) (*models.CategoryResponse, error) {
	category, err := s.categoryRepo.GetByID(id)
	if err != nil {
		return nil, &apperrors.ErrNotFound{Resource: "Category", ID: id}
	}

	updateData := models.Category{Name: name}
	if err := s.categoryRepo.Update(category, updateData); err != nil {
		return nil, err
	}

	return &models.CategoryResponse{
		ID:   id,
		Name: name, // Return the new name directly
	}, nil
}

func (s *categoryService) Delete(id uint) error {
	category, err := s.categoryRepo.GetByID(id)
	if err != nil {
		return &apperrors.ErrNotFound{Resource: "Category", ID: id}
	}
	return s.categoryRepo.Delete(category)
}