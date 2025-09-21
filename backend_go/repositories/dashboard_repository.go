package repositories

import (
	"frbktg/backend_go/models"
	"time"

	"github.com/pkg/errors"
	"gorm.io/gorm"
)

type DashboardRepository interface {
	WithTx(tx *gorm.DB) DashboardRepository
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

func (r *gormDashboardRepository) WithTx(tx *gorm.DB) DashboardRepository {
	return &gormDashboardRepository{db: tx}
}

func (r *gormDashboardRepository) CountTotalUsers() (int64, error) {
	var totalUsers int64
	err := r.db.Model(&models.BotUser{}).Where("is_deleted = ?", false).Count(&totalUsers).Error
	return totalUsers, errors.WithStack(err)
}

func (r *gormDashboardRepository) CountUsersWithPurchases() (int64, error) {
	var usersWithPurchases int64
	err := r.db.Model(&models.Order{}).Distinct("user_id").Count(&usersWithPurchases).Error
	return usersWithPurchases, errors.WithStack(err)
}

func (r *gormDashboardRepository) CountAvailableProducts() (int64, error) {
	var availableProducts int64
	err := r.db.Model(&models.StockMovement{}).
		Select("product_id").
		Group("product_id").
		Having("sum(quantity) > 0").
		Count(&availableProducts).Error
	return availableProducts, errors.WithStack(err)
}

func (r *gormDashboardRepository) GetSalesCountForPeriod(start, end time.Time) (int64, error) {
	var productsSold int64
	err := r.db.Model(&models.Order{}).Where(
		"created_at >= ? AND created_at < ?", start, end.AddDate(0, 0, 1),
	).Count(&productsSold).Error
	return productsSold, errors.WithStack(err)
}

func (r *gormDashboardRepository) GetTotalRevenueForPeriod(start, end time.Time) (float64, error) {
	var totalRevenue float64
	err := r.db.Model(&models.Order{}).Where("created_at >= ? AND created_at < ?", start, end.AddDate(0, 0, 1)).Select("COALESCE(sum(amount), 0)").
		Row().Scan(&totalRevenue)
	if err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		return 0, errors.WithStack(err)
	}
	return totalRevenue, nil
}
