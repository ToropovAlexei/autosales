package repositories

import (
	"frbktg/backend_go/models"
	"gorm.io/gorm"
	"gorm.io/gorm/clause"
)

type StoreBalanceRepository interface {
	GetStoreBalance(tx *gorm.DB) (*models.StoreBalance, error)
	UpdateStoreBalance(tx *gorm.DB, amount float64) error
}

type gormStoreBalanceRepository struct {
	db *gorm.DB
}

func NewGormStoreBalanceRepository(db *gorm.DB) StoreBalanceRepository {
	return &gormStoreBalanceRepository{db: db}
}

func (r *gormStoreBalanceRepository) GetStoreBalance(tx *gorm.DB) (*models.StoreBalance, error) {
	var balance models.StoreBalance
	db := r.db
	if tx != nil {
		db = tx
	}
	err := db.First(&balance).Error
	if err != nil {
		if err == gorm.ErrRecordNotFound {
			// Create if not exists
			zeroBalance := models.StoreBalance{CurrentBalance: 0}
			if err := db.Create(&zeroBalance).Error; err != nil {
				return nil, err
			}
			return &zeroBalance, nil
		}
		return nil, err
	}
	return &balance, nil
}

func (r *gormStoreBalanceRepository) UpdateStoreBalance(tx *gorm.DB, amount float64) error {
	db := r.db
	if tx != nil {
		db = tx
	}

	// Ensure the record exists before updating
	_, err := r.GetStoreBalance(tx)
	if err != nil {
		return err
	}

	// Atomically update the balance
	return db.Model(&models.StoreBalance{}).Clauses(clause.Returning{}).Where("id = ?", 1).Update("current_balance", gorm.Expr("current_balance + ?", amount)).Error
}
