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
	GetSalesOverTime(start, end time.Time) ([]models.TimeSeriesData, error)
	GetUsersOverTime(start, end time.Time) ([]models.TimeSeriesData, error)
	CountTotalUsersForPeriod(start, end time.Time) (int64, error)
	CountUsersWithPurchasesForPeriod(start, end time.Time) (int64, error)
	CountProductsSoldForPeriod(start, end time.Time) (int64, error)
	GetTopProducts(limit int) ([]models.Product, error)
	GetSalesByCategory() ([]models.CategorySales, error)
	GetRevenueOverTime(start, end time.Time) ([]models.TimeSeriesData, error)
	GetUsersWithPurchasesOverTime(start, end time.Time) ([]models.TimeSeriesData, error)
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

func (r *gormDashboardRepository) GetSalesOverTime(start, end time.Time) ([]models.TimeSeriesData, error) {
	var results []models.TimeSeriesData
	err := r.db.Model(&models.Order{}).
		Select("DATE(created_at) as date, COUNT(*) as value").
		Where("created_at >= ? AND created_at < ?", start, end.AddDate(0, 0, 1)).
		Group("DATE(created_at)").
		Order("date").
		Scan(&results).Error
	return results, errors.WithStack(err)
}

func (r *gormDashboardRepository) GetUsersOverTime(start, end time.Time) ([]models.TimeSeriesData, error) {
	var results []models.TimeSeriesData
	err := r.db.Model(&models.BotUser{}).
		Select("DATE(created_at) as date, COUNT(*) as value").
		Where("created_at >= ? AND created_at < ? AND is_deleted = ?", start, end.AddDate(0, 0, 1), false).
		Group("DATE(created_at)").
		Order("date").
		Scan(&results).Error
	return results, errors.WithStack(err)
}

func (r *gormDashboardRepository) CountTotalUsersForPeriod(start, end time.Time) (int64, error) {
	var totalUsers int64
	err := r.db.Model(&models.BotUser{}).Where("created_at >= ? AND created_at < ? AND is_deleted = ?", start, end, false).Count(&totalUsers).Error
	return totalUsers, errors.WithStack(err)
}

func (r *gormDashboardRepository) CountUsersWithPurchasesForPeriod(start, end time.Time) (int64, error) {
	var usersWithPurchases int64
	err := r.db.Model(&models.Order{}).Where("created_at >= ? AND created_at < ?", start, end).Distinct("user_id").Count(&usersWithPurchases).Error
	return usersWithPurchases, errors.WithStack(err)
}


func (r *gormDashboardRepository) CountProductsSoldForPeriod(start, end time.Time) (int64, error) {
	var productsSold int64
	err := r.db.Model(&models.Order{}).Where("created_at >= ? AND created_at < ?", start, end).Count(&productsSold).Error
	return productsSold, errors.WithStack(err)
}

func (r *gormDashboardRepository) GetTopProducts(limit int) ([]models.Product, error) {
	var products []models.Product
	err := r.db.Model(&models.Order{}).
		Select("products.*, sum(orders.amount) as total_revenue").
		Joins("join products on products.id = orders.product_id").
		Group("products.id").
		Order("total_revenue desc").
		Limit(limit).
		Find(&products).Error
	return products, errors.WithStack(err)
}

func (r *gormDashboardRepository) GetSalesByCategory() ([]models.CategorySales, error) {
	var results []models.CategorySales
	err := r.db.Model(&models.Order{}).
		Select("categories.name as category_name, sum(orders.amount) as total_sales").
		Joins("join products on products.id = orders.product_id").
		Joins("join categories on categories.id = products.category_id").
		Group("categories.name").
		Find(&results).Error
	return results, errors.WithStack(err)
}

func (r *gormDashboardRepository) GetRevenueOverTime(start, end time.Time) ([]models.TimeSeriesData, error) {
	var results []models.TimeSeriesData
	err := r.db.Model(&models.Order{}).
		Select("DATE(created_at) as date, SUM(amount) as value").
		Where("created_at >= ? AND created_at < ?", start, end.AddDate(0, 0, 1)).
		Group("DATE(created_at)").
		Order("date").
		Scan(&results).Error
	return results, errors.WithStack(err)
}

func (r *gormDashboardRepository) GetUsersWithPurchasesOverTime(start, end time.Time) ([]models.TimeSeriesData, error) {
	var results []models.TimeSeriesData
	err := r.db.Model(&models.Order{}).
		Select("DATE(created_at) as date, COUNT(DISTINCT user_id) as value").
		Where("created_at >= ? AND created_at < ?", start, end.AddDate(0, 0, 1)).
		Group("DATE(created_at)").
		Order("date").
		Scan(&results).Error
	return results, errors.WithStack(err)
}
