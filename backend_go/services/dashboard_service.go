package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"time"

	"golang.org/x/sync/errgroup"
)

type DashboardStats struct {
	TotalUsers         int64 `json:"total_users"`
	UsersWithPurchases int64 `json:"users_with_purchases"`
	AvailableProducts  int64 `json:"available_products"`
}

type TimeSeriesDashboardData struct {
	Sales                 *SalesOverTime           `json:"sales"`
	SalesChart            []models.TimeSeriesData `json:"sales_chart"`
	UsersChart            []models.TimeSeriesData `json:"users_chart"`
	RevenueChart          []models.TimeSeriesData `json:"revenue_chart"`
	UsersWithPurchasesChart []models.TimeSeriesData `json:"users_with_purchases_chart"`
}

type SalesOverTime struct {
	ProductsSold int64   `json:"products_sold"`
	TotalRevenue float64 `json:"total_revenue"`
}

type DashboardService interface {
	GetDashboardStats() (*models.DashboardOverview, error)
	GetTimeSeriesDashboardData(start, end time.Time) (*TimeSeriesDashboardData, error)
	GetTopProducts() ([]models.Product, error)
	GetSalesByCategory() ([]models.CategorySales, error)
}

type dashboardService struct {
	dashboardRepo repositories.DashboardRepository
}

func NewDashboardService(dashboardRepo repositories.DashboardRepository) DashboardService {
	return &dashboardService{dashboardRepo: dashboardRepo}
}

func fillMissingDates(start, end time.Time, data []models.TimeSeriesData) []models.TimeSeriesData {
	dateMap := make(map[string]int64)
	for _, d := range data {
		dateMap[d.Date.Format("2006-01-02")] = d.Value
	}

	var filledData []models.TimeSeriesData
	for d := start; !d.After(end); d = d.AddDate(0, 0, 1) {
		dateStr := d.Format("2006-01-02")
		value, ok := dateMap[dateStr]
		if !ok {
			value = 0
		}
		filledData = append(filledData, models.TimeSeriesData{Date: d, Value: value})
	}
	return filledData
}

func (s *dashboardService) GetDashboardStats() (*models.DashboardOverview, error) {
	var g errgroup.Group

	overview := &models.DashboardOverview{}

	g.Go(func() error {
		var err error
		overview.TotalUsers, err = s.dashboardRepo.CountTotalUsers()
		return err
	})

	g.Go(func() error {
		var err error
		overview.UsersWithPurchases, err = s.dashboardRepo.CountUsersWithPurchases()
		return err
	})

	g.Go(func() error {
		var err error
		overview.AvailableProducts, err = s.dashboardRepo.CountAvailableProducts()
		return err
	})

	end := time.Now()
	start := end.AddDate(0, 0, -30)
	prevEnd := start
	prevStart := prevEnd.AddDate(0, 0, -30)

	g.Go(func() error {
		current, err := s.dashboardRepo.CountTotalUsersForPeriod(start, end)
		if err != nil {
			return err
		}
		previous, err := s.dashboardRepo.CountTotalUsersForPeriod(prevStart, prevEnd)
		if err != nil {
			return err
		}
		overview.TotalUsers30Days.Value = current
		if previous == 0 {
			overview.TotalUsers30Days.Trend = 0
		} else {
			overview.TotalUsers30Days.Trend = (float64(current) - float64(previous)) / float64(previous) * 100
		}
		return nil
	})

	g.Go(func() error {
		current, err := s.dashboardRepo.CountUsersWithPurchasesForPeriod(start, end)
		if err != nil {
			return err
		}
		previous, err := s.dashboardRepo.CountUsersWithPurchasesForPeriod(prevStart, prevEnd)
		if err != nil {
			return err
		}
		overview.UsersWithPurchases30Days.Value = current
		if previous == 0 {
			overview.UsersWithPurchases30Days.Trend = 0
		} else {
			overview.UsersWithPurchases30Days.Trend = (float64(current) - float64(previous)) / float64(previous) * 100
		}
		return nil
	})

	g.Go(func() error {
		current, err := s.dashboardRepo.CountProductsSoldForPeriod(start, end)
		if err != nil {
			return err
		}
		previous, err := s.dashboardRepo.CountProductsSoldForPeriod(prevStart, prevEnd)
		if err != nil {
			return err
		}
		overview.ProductsSold30Days.Value = current
		if previous == 0 {
			overview.ProductsSold30Days.Trend = 0
		} else {
			overview.ProductsSold30Days.Trend = (float64(current) - float64(previous)) / float64(previous) * 100
		}
		return nil
	})

	if err := g.Wait(); err != nil {
		return nil, err
	}

	return overview, nil
}

func (s *dashboardService) GetTimeSeriesDashboardData(start, end time.Time) (*TimeSeriesDashboardData, error) {
	var g errgroup.Group

	data := &TimeSeriesDashboardData{}

	g.Go(func() error {
		var g2 errgroup.Group
		salesData := &SalesOverTime{}
		g2.Go(func() error {
			var err error
			salesData.ProductsSold, err = s.dashboardRepo.GetSalesCountForPeriod(start, end)
			return err
		})
		g2.Go(func() error {
			var err error
			salesData.TotalRevenue, err = s.dashboardRepo.GetTotalRevenueForPeriod(start, end)
			return err
		})
		if err := g2.Wait(); err != nil {
			return err
		}
		data.Sales = salesData
		return nil
	})

	g.Go(func() error {
		var err error
		raw, err := s.dashboardRepo.GetSalesOverTime(start, end)
		data.SalesChart = fillMissingDates(start, end, raw)
		return err
	})

	g.Go(func() error {
		var err error
		raw, err := s.dashboardRepo.GetUsersOverTime(start, end)
		data.UsersChart = fillMissingDates(start, end, raw)
		return err
	})

	g.Go(func() error {
		var err error
		raw, err := s.dashboardRepo.GetRevenueOverTime(start, end)
		data.RevenueChart = fillMissingDates(start, end, raw)
		return err
	})

	g.Go(func() error {
		var err error
		raw, err := s.dashboardRepo.GetUsersWithPurchasesOverTime(start, end)
		data.UsersWithPurchasesChart = fillMissingDates(start, end, raw)
		return err
	})

	if err := g.Wait(); err != nil {
		return nil, err
	}

	return data, nil
}

func (s *dashboardService) GetTopProducts() ([]models.Product, error) {
	return s.dashboardRepo.GetTopProducts(5)
}

func (s *dashboardService) GetSalesByCategory() ([]models.CategorySales, error) {
	return s.dashboardRepo.GetSalesByCategory()
}
