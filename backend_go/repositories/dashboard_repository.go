package repositories

import (
	"errors"
	"frbktg/backend_go/models"
	"time"

	"gorm.io/gorm"
)

type DashboardRepository interface {
	CountTotalUsers() (int64, error)
	CountUsersWithPurchases() (int64, error)
	CountAvailableProducts() (int64, error)
	GetSalesCountForPeriod(start, end time.Time) (int64, error)
	GetTotalRevenueForPeriod(start, end time.Time) (float64, error)
}

type gormDashboardRepository struct {
	db *gorm.DB
}

func NewDashboardRepository(db *gorm.DB) DashboardRepository {
	return &gormDashboardRepository{db: db}
}

func (r *gormDashboardRepository) CountTotalUsers() (int64, error) {
	var totalUsers int64
	err := r.db.Model(&models.BotUser{}).Where("is_deleted = ?", false).Count(&totalUsers).Error
	return totalUsers, err
}

func (r *gormDashboardRepository) CountUsersWithPurchases() (int64, error) {
	var usersWithPurchases int64
	err := r.db.Model(&models.Order{}).Distinct("user_id").Count(&usersWithPurchases).Error
	return usersWithPurchases, err
}

func (r *gormDashboardRepository) CountAvailableProducts() (int64, error) {
	var productIDs []uint
	if err := r.db.Model(&models.Product{}).Pluck("id", &productIDs).Error; err != nil {
		return 0, err
	}

	var availableProducts int64
	for _, id := range productIDs {
		var stock int64
		if err := r.db.Model(&models.StockMovement{}).Where("product_id = ?", id).Select("sum(quantity)").
			Row().Scan(&stock); err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
			return 0, err
		}
		if stock > 0 {
			availableProducts++
		}
	}
	return availableProducts, nil
}

func (r *gormDashboardRepository) GetSalesCountForPeriod(start, end time.Time) (int64, error) {
	var productsSold int64
	err := r.db.Model(&models.Order{}).Where(
		"created_at >= ? AND created_at < ?", start, end.AddDate(0, 0, 1),
	).Count(&productsSold).Error
	return productsSold, err
}

func (r *gormDashboardRepository) GetTotalRevenueForPeriod(start, end time.Time) (float64, error) {
	var totalRevenue float64
	if err := r.db.Model(&models.Order{}).Where("created_at >= ? AND created_at < ?", start, end.AddDate(0, 0, 1)).Select("sum(amount)").
		Row().Scan(&totalRevenue); err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		return 0, err
	}
	return totalRevenue, nil
}
