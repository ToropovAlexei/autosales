package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type StockRepository interface {
	WithTx(tx *gorm.DB) StockRepository
	GetStockMovements(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.StockMovement], error)
}

type gormStockRepository struct {
	db *gorm.DB
}

func NewStockRepository(db *gorm.DB) StockRepository {
	return &gormStockRepository{db: db}
}

func (r *gormStockRepository) WithTx(tx *gorm.DB) StockRepository {
	return &gormStockRepository{db: tx}
}

func (r *gormStockRepository) GetStockMovements(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.StockMovement], error) {
	db := r.db.Model(&models.StockMovement{})
	db = ApplyFilters[models.StockMovement](db, filters)

	paginatedResult, err := ApplyPagination[models.StockMovement](db, page)
	if err != nil {
		return nil, err
	}

	return paginatedResult, nil
}
