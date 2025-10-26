package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type StockRepository interface {
	WithTx(tx *gorm.DB) StockRepository
	GetStockMovements(filters []models.Filter) ([]models.StockMovement, error)
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

func (r *gormStockRepository) GetStockMovements(filters []models.Filter) ([]models.StockMovement, error) {
	var movements []models.StockMovement
	db := r.db.Model(&models.StockMovement{})
	db = ApplyFilters[models.StockMovement](db, filters)
	if err := db.Order("created_at desc").Find(&movements).Error; err != nil {
		return nil, err
	}
	return movements, nil
}
