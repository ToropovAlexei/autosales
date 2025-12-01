package repositories

import (
	"frbktg/backend_go/models"
	"gorm.io/gorm"
)

type StoreBalanceRepository interface {
	GetStoreBalance(tx *gorm.DB) (*models.StoreBalance, error)
	UpdateStoreBalance(tx *gorm.DB, amount float64) error
	SetStoreBalance(tx *gorm.DB, amount float64) error
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
	// Use ID 1 as it's a single-row table for the global store balance.
	err := db.First(&balance, 1).Error
	if err != nil {
		if err == gorm.ErrRecordNotFound {
			// Create if not exists
			zeroBalance := models.StoreBalance{CurrentBalance: 0}
			zeroBalance.ID = 1
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
	if _, err := r.GetStoreBalance(tx); err != nil {
		return err
	}

	// Atomically update the balance
	return db.Model(&models.StoreBalance{}).Where("id = ?", 1).Update("current_balance", gorm.Expr("current_balance + ?", amount)).Error
}

func (r *gormStoreBalanceRepository) SetStoreBalance(tx *gorm.DB, amount float64) error {
    db := r.db
    if tx != nil {
        db = tx
    }
    // First, ensure the balance record exists.
    if _, err := r.GetStoreBalance(tx); err != nil {
        return err
    }

    // Now, set the balance to the specified amount.
    return db.Model(&models.StoreBalance{}).Where("id = ?", 1).Update("current_balance", amount).Error
}
